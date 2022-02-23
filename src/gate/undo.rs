use std::collections::hash_set::HashSet;
use crate::gate::{
    core::{Name, *},
    graphics::{clk::*, light_bulb::*, toggle_switch::*},
    serialize::*,
};
use bevy::prelude::*;

pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UndoEvent>()
            .add_event::<ReconnectGates>()
            .insert_resource(UndoStack {
                undo: Vec::new(),
                redo: Vec::new(),
            })
            .add_system(reconnect_gates_event_system.before("handle_undo"))
            // Not pretty but this system must run after the disconnect
            // system to prevent program crashes due to data races.
            .add_system(handle_undo_event_system.label("handle_undo").after("disconnect"))
            .add_system(listen_for_new_connections_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UndoEvent {
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub enum Action {
    Insert((Vec<NodusComponent>, HashSet<(ConnInfo, ConnInfo)>)),
    Remove(Vec<Entity>),
    InsertConnection(Entity),
    RemoveConnection(Entity),
}

#[derive(Debug, Clone)]
pub struct UndoStack {
    pub undo: Vec<Action>,
    pub redo: Vec<Action>,
}

pub fn handle_undo_event_system(
    mut commands: Commands,
    mut stack: ResMut<UndoStack>,
    mut ev_undo: EventReader<UndoEvent>,
    server: Res<AssetServer>,
    q_node: Query<(
        Entity,
        &Name,
        Option<&Inputs>,
        Option<&Outputs>,
        Option<&Targets>,
        Option<&Clk>,
        &Transform,
        &NodeType,
    )>,
    children: Query<&Children>,
    q_connectors: Query<&Connections>,
    q_line: Query<(Entity, &ConnectionLine)>,
    q_parent: Query<&Parent>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    mut ev_conn: EventWriter<ReconnectGates>,
) {
    let font: Handle<Font> = server.load("fonts/hack.bold.ttf");

    for ev in ev_undo.iter() {
        match ev {
            UndoEvent::Undo => {
                if let Some(action) = stack.undo.pop() {
                    match action {
                        Action::Insert(mut e) => {
                            if let Some(entities) = insert(
                                &mut commands, 
                                font.clone(), 
                                e.0.clone(),
                            ) {
                                if e.0.len() == entities.len() {
                                    for i in 0..e.0.len() {
                                        replace_entity_id(
                                            e.0[i].id, 
                                            entities[i], 
                                            &mut stack
                                        );
                                        replace_entity_id_(
                                            e.0[i].id, 
                                            entities[i], 
                                            &mut e.1
                                        );
                                    }
                                }
                                ev_conn.send(ReconnectGates(e.1));
                                stack.redo.push(Action::Remove(entities));
                            }
                        }
                        Action::Remove(entities) => {
                            if let Some(nc) = remove(
                                &mut commands, 
                                entities, 
                                &q_node, 
                                &children, 
                                &q_connectors,
                                &q_line,
                                &q_parent,
                                &mut ev_disconnect
                            ) {
                                stack.redo.push(Action::Insert(nc));
                            }
                        }
                        Action::RemoveConnection(c) => {
                            ev_disconnect.send(DisconnectEvent {
                                connection: c,
                                in_parent: None,
                            });
                        }
                        _ => { }
                    }
                }
            }
            UndoEvent::Redo => {
                if let Some(action) = stack.redo.pop() {
                    match action {
                        Action::Insert(mut e) => {
                            if let Some(entities) = insert(
                                &mut commands, 
                                font.clone(), 
                                e.0.clone()
                            ) {
                                if e.0.len() == entities.len() {
                                    for i in 0..e.0.len() {
                                        replace_entity_id(
                                            e.0[i].id, 
                                            entities[i], 
                                            &mut stack
                                        );
                                        replace_entity_id_(
                                            e.0[i].id, 
                                            entities[i], 
                                            &mut e.1
                                        );
                                    }
                                }

                                ev_conn.send(ReconnectGates(e.1));
                                stack.undo.push(Action::Remove(entities));
                            }
                        },
                        Action::Remove(entities) => {
                            if let Some(nc) = remove(
                                &mut commands, 
                                entities, 
                                &q_node, 
                                &children, 
                                &q_connectors, 
                                &q_line,
                                &q_parent,
                                &mut ev_disconnect
                            ) {
                                stack.undo.push(Action::Insert(nc));
                            }
                        },
                        Action::RemoveConnection(c) => {
                            ev_disconnect.send(DisconnectEvent {
                                connection: c,
                                in_parent: None,
                            });
                        },
                        _ => { }
                    }
                }
            }
        }
    }
}

pub struct ReconnectGates(pub HashSet<(ConnInfo, ConnInfo)>);

fn reconnect_gates_event_system(
    mut ev_conn: EventReader<ReconnectGates>,
    mut cev: EventWriter<ConnectEvent>,
    q_children: Query<&Children>,
    q_conn: Query<(Entity, &Connector)>,
) {
    for conns in ev_conn.iter() { 
        'rg_loop: for (lhs, rhs) in conns.0.iter() {
            if let Ok(lhs_children) = q_children.get(lhs.entity) {
                if let Ok(rhs_children) = q_children.get(rhs.entity) {
                    for &lhs_child in lhs_children.iter() {
                        if let Ok((lhs_e, lhs_con)) = q_conn.get(lhs_child) {
                            if lhs_con.index == lhs.index &&
                                lhs_con.ctype == ConnectorType::Out {
                                for &rhs_child in rhs_children.iter() {
                                    if let Ok((rhs_e, rhs_con)) = q_conn.get(rhs_child) {
                                        if rhs_con.index == rhs.index &&
                                            rhs_con.ctype == ConnectorType::In {
                                            cev.send(ConnectEvent {
                                                output: lhs_e,
                                                output_index: lhs.index,
                                                input: rhs_e,
                                                input_index: rhs.index,
                                                signal_success: false,
                                            });
                                            continue 'rg_loop;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn replace_entity_id_(old: Entity, new: Entity, con: &mut HashSet<(ConnInfo, ConnInfo)>) {
    let mut tmp = Vec::new();

    for mut t in con.iter() {
        if t.0.entity == old && t.1.entity != old {
            tmp.push((
                t.clone(),
                (
                    ConnInfo { entity: new, index: t.0.index },
                    t.1.clone()
                )
            ));
        } else if t.0.entity != old && t.1.entity == old {
            tmp.push((
                t.clone(),
                (
                    t.0.clone(),
                    ConnInfo { entity: new, index: t.1.index },
                )
            ));
        } else if t.0.entity == old && t.1.entity == old {
            tmp.push((
                t.clone(),
                (
                    ConnInfo { entity: new, index: t.0.index },
                    ConnInfo { entity: new, index: t.1.index },
                )
            ));
        }
    }

    for (o, n) in tmp.drain(..) {
        con.remove(&o);
        con.insert(n);
    }
}

fn replace_entity_id2_(old: Entity, new: Entity, v: &mut Vec<Entity>) {
    for i in 0..v.len() {
        if v[i] == old {
            v[i] = new;
        }
    }
}

/// Replace the given `old` entity id with the `new` entity id within the
/// target maps of all NodusComponent's of the undo/redo stack.
fn replace_entity_id(old: Entity, new: Entity, stack: &mut ResMut<UndoStack>) {
    for mut action in &mut stack.undo {
        match action {
            Action::Insert(ncs) => {
                replace_entity_id_(old, new, &mut ncs.1);
            },
            Action::Remove(ref mut es) => { 
                replace_entity_id2_(old, new, es);
            },
            _ => { }
        }
    }

    for mut action in &mut stack.redo {
        match action {
            Action::Insert(ncs) => {
                replace_entity_id_(old, new, &mut ncs.1);
            },
            Action::Remove(ref mut es) => { 
                replace_entity_id2_(old, new, es);
            },
            _ => { }
        }
    }
}

fn insert(
    commands: &mut Commands, 
    font: Handle<Font>, 
    components: Vec<NodusComponent>,
) -> Option<Vec<Entity>> {
    let mut res = Vec::new();

    for e in components {
        let entity = match e.ntype {
            NodeType::And => {
                Some(
                    Gate::and_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::Nand => {
                Some(
                    Gate::nand_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::Or => {
                Some(
                    Gate::or_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::Nor => {
                Some(
                    Gate::nor_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::Xor => {
                Some(
                    Gate::xor_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::Xnor => { None }
            NodeType::Not => {
                Some(
                    Gate::not_gate_bs_(
                        commands,
                        e.position,
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::HighConst => {
                Some(Gate::high_const(commands, e.position, font.clone()))
            }
            NodeType::LowConst => {
                Some(Gate::low_const(commands, e.position, font.clone()))
            }
            NodeType::ToggleSwitch => {
                if let Some(NodeState::ToggleSwitch(state)) = e.state {
                    Some(ToggleSwitch::new(commands, e.position, state))
                } else { None }
            }
            NodeType::Clock => {
                if let Some(NodeState::Clock(x1, x2, x3)) = e.state {
                    Some(Clk::spawn(commands, e.position, x1, x2, x3))
                } else { None }
            }
            NodeType::LightBulb => {
                if let Some(NodeState::LightBulb(state)) = e.state {
                    Some(LightBulb::spawn(commands, e.position, state))
                } else { None }
            }
        };

        if let Some(entity) = entity {
            res.push(entity);
        }
    }

    if res.len() > 0 { Some(res) }
    else { None }
}

pub fn remove(
    commands: &mut Commands, 
    entities: Vec<Entity>,
    q_node: &Query<(
        Entity,
        &Name,
        Option<&Inputs>,
        Option<&Outputs>,
        Option<&Targets>,
        Option<&Clk>,
        &Transform,
        &NodeType,
    )>,
    children: &Query<&Children>,
    q_connectors: &Query<&Connections>,
    q_line: &Query<(Entity, &ConnectionLine)>,
    q_parent: &Query<&Parent>,
    ev_disconnect: &mut EventWriter<DisconnectEvent>,
) -> Option<(Vec<NodusComponent>, HashSet<(ConnInfo, ConnInfo)>)> {
    let mut res = Vec::new();
    let mut con = HashSet::new();

    for e in entities {
        if let Ok((e, n, ip, op, t, clk, tr, nt)) = q_node.get(e) {
            let i = if let Some(i) = ip {
                Some(i.len())
            } else {
                None
            };
            let o = if let Some(o) = op {
                Some(o.len())
            } else {
                None
            };
            let t = if let Some(t) = t {
                Some(t.clone())
            } else {
                None
            };

            let state = match &nt {
                NodeType::ToggleSwitch => {
                    Some(NodeState::ToggleSwitch(op.unwrap()[0]))
                }
                NodeType::Clock => {
                    let clk = clk.unwrap();
                    Some(NodeState::Clock(clk.0, clk.1, op.unwrap()[0]))
                }
                NodeType::LightBulb => {
                    Some(NodeState::LightBulb(ip.unwrap()[0]))
                }
                _ => None,
            };
            
            let nc = NodusComponent {
                    id: e,
                    name: n.0.to_string(),
                    inputs: i,
                    outputs: o,
                    targets: t,
                    position: Vec2::new(tr.translation.x, tr.translation.y),
                    rotation: Some(tr.rotation),
                    ntype: nt.clone(),
                    state: state,
            };

            if let Ok(children) = children.get(e) {
                for &child in children.iter() {
                    if let Ok(conns) = q_connectors.get(child) {
                        for &connection in conns.iter() {
                            if let Ok((entity, line)) = q_line.get(connection) {
                                if let Ok(parent1) = q_parent.get(line.output.entity) {
                                    if let Ok(parent2) = q_parent.get(line.input.entity) {
                                        // By using a HashSet we dont need to check
                                        // if a connection already exists.
                                        let c = (
                                            // ConnInfo usually holds a reference to a connector
                                            // and not the gate itself, but we gonna reuse it here.
                                            ConnInfo { entity: parent1.0, index: line.output.index },
                                            ConnInfo { entity: parent2.0, index: line.input.index }
                                        );
                                        con.insert(c);
                                        eprintln!("line");
                                    }
                                }
                            }

                            ev_disconnect.send(DisconnectEvent {
                                connection,
                                in_parent: Some(e),
                            });
                        }
                    }
                }
            }

            commands.entity(e).despawn_recursive();
            res.push(nc);
        }
    }

    if res.len() > 0 { Some((res, con)) }
    else { None }
}

fn listen_for_new_connections_system(
    mut ev_est: EventReader<NewConnectionEstablishedEvent>,
    mut stack: ResMut<UndoStack>,
) {
    for ev in ev_est.iter() {
        stack.undo.push(Action::RemoveConnection(ev.id));
        stack.redo.clear();
    }
}
