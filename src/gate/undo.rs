use std::collections::hash_set::HashSet;
use crate::gate::{
    core::{Name, *},
    graphics::{clk::*, light_bulb::*, toggle_switch::*, segment_display::*},
    serialize::*,
};
use bevy::prelude::*;
use crate::GameState;

pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UndoEvent>()
            .add_event::<ReconnectGates>()
            .add_event::<DisconnectEventUndo>()
            .insert_resource(UndoStack {
                undo: Vec::new(),
                redo: Vec::new(),
            })
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("undo")
                    .with_system(reconnect_gates_event_system.before("handle_undo"))
                    // Not pretty but this system must run after the disconnect
                    // system to prevent program crashes due to data races.
                    .with_system(handle_undo_event_system.label("handle_undo").after("disconnect"))
                    .with_system(listen_for_new_connections_system)
                    // Alot of systems run after disconnect to prevent Segfaults,
                    // i.e. we must run this system also before the others.
                    .with_system(disconnect_event_system_undo.before("disconnect").after("draw_line"))
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UndoEvent {
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub enum Action {
    Insert((Vec<NodusComponent>, HashSet<(ConnInfo, ConnInfo, Entity)>)),
    Remove(Vec<Entity>),
    InsertConnection((ConnInfo, ConnInfo, Entity)),
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
    mut ev_disconnect_undo: EventWriter<DisconnectEventUndo>,
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
                                ev_conn.send(ReconnectGates(e.1, None));
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
                            ev_disconnect_undo.send(DisconnectEventUndo {
                                connection: c,
                                in_parent: None,
                                action: UndoEvent::Redo,
                            });
                        }
                        Action::InsertConnection(con) => {
                            let mut h = HashSet::new();
                            h.insert(con);
                            ev_conn.send(ReconnectGates(h, Some(UndoEvent::Redo)));
                        },
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

                                ev_conn.send(ReconnectGates(e.1, None));
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
                            ev_disconnect_undo.send(DisconnectEventUndo {
                                connection: c,
                                in_parent: None,
                                action: UndoEvent::Undo,
                            });
                        },
                        Action::InsertConnection(con) => {
                            let mut h = HashSet::new();
                            h.insert(con);
                            ev_conn.send(ReconnectGates(h, Some(UndoEvent::Undo)));
                        },
                        _ => { }
                    }
                }
            }
        }
    }
}

pub struct ReconnectGates(
    pub HashSet<(ConnInfo, ConnInfo, Entity)>,
    Option<UndoEvent>
);

fn reconnect_gates_event_system(
    mut ev_conn: EventReader<ReconnectGates>,
    q_children: Query<&Children>,
    q_conn: Query<(Entity, &Connector)>,

    mut commands: Commands,
    mut q_conns: Query<(&Parent, &mut Connections), ()>,
    mut q_parent: Query<&mut Targets>,
    
    mut stack: ResMut<UndoStack>,
) {
    for conns in ev_conn.iter() { 
        for conn in conns.0.iter() {
            if let Ok(e) = reconnect_gates(
                &q_children,
                &q_conn,
                &mut commands,
                &mut q_conns,
                &mut q_parent,
                &mut stack,
                conn
            ) {
                if let Some(action) = conns.1 {
                    match action {
                        UndoEvent::Undo => {
                            stack.undo.push(Action::RemoveConnection(e));
                        },
                        UndoEvent::Redo => {
                            stack.redo.push(Action::RemoveConnection(e));
                        }
                    }
                }
            }
        }
    }
}

fn reconnect_gates(
    q_children: &Query<&Children>,
    q_conn: &Query<(Entity, &Connector)>,

    commands: &mut Commands,
    q_conns: &mut Query<(&Parent, &mut Connections), ()>,
    q_parent: &mut Query<&mut Targets>,
    
    stack: &mut ResMut<UndoStack>,
    (lhs, rhs, old_id): &(ConnInfo, ConnInfo, Entity),
) -> Result<Entity, ()> {
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
                                    let new_id = connect(
                                        commands,
                                        q_conns,
                                        q_parent,
                                        &ConnectEvent {
                                            output: lhs_e,
                                            output_index: lhs.index,
                                            input: rhs_e,
                                            input_index: rhs.index,
                                            signal_success: false,
                                        }
                                    );

                                    replace_connection_entity_id_(
                                        *old_id,
                                        new_id,
                                        stack,
                                    );
                                    
                                    return Ok(new_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(())
}

fn replace_entity_id_(old: Entity, new: Entity, con: &mut HashSet<(ConnInfo, ConnInfo, Entity)>) {
    let mut tmp = Vec::new();

    for t in con.iter() {
        if let Some(t_new) = replace_entity_id3_(old, new, t.clone()) {
            tmp.push((t.clone(), t_new));
        }
    }

    for (o, n) in tmp.drain(..) {
        con.remove(&o);
        con.insert(n);
    }
}

fn replace_entity_id3_(old: Entity, new: Entity, t: (ConnInfo, ConnInfo, Entity)) -> Option<(ConnInfo, ConnInfo, Entity)> {
    if t.0.entity == old && t.1.entity != old {
        Some(
            (
                ConnInfo { entity: new, index: t.0.index },
                t.1.clone(),
                t.2
            )
        )
    } else if t.0.entity != old && t.1.entity == old {
        Some(
            (
                t.0.clone(),
                ConnInfo { entity: new, index: t.1.index },
                t.2
            )
        )
    } else if t.0.entity == old && t.1.entity == old {
        Some(
            (
                ConnInfo { entity: new, index: t.0.index },
                ConnInfo { entity: new, index: t.1.index },
                t.2,
            )
        )
    } else {
        None
    }
}

fn replace_entity_id2_(old: Entity, new: Entity, v: &mut Vec<Entity>) {
    for i in 0..v.len() {
        if v[i] == old {
            v[i] = new;
        }
    }
}

fn replace_connection_entity_id_(old: Entity, new: Entity, stack: &mut ResMut<UndoStack>) {
    for action in &mut stack.undo {
        match action {
            Action::RemoveConnection(ref mut id) => { 
                if *id == old { *id = new; }
            },
            _ => { }
        }
    }

    for action in &mut stack.redo {
        match action {
            Action::RemoveConnection(ref mut id) => { 
                if *id == old { *id = new; }
            },
            _ => { }
        }
    }
}

/// Replace the given `old` entity id with the `new` entity id within the
/// target maps of all NodusComponent's of the undo/redo stack.
fn replace_entity_id(old: Entity, new: Entity, stack: &mut ResMut<UndoStack>) {
    for action in &mut stack.undo {
        match action {
            Action::Insert(ncs) => {
                replace_entity_id_(old, new, &mut ncs.1);
            },
            Action::Remove(ref mut es) => { 
                replace_entity_id2_(old, new, es);
            },
            Action::InsertConnection(ref mut con) => {
                if let Some(c_new) = replace_entity_id3_(old, new, con.clone()) {
                    *con = c_new;
                }
            },
            _ => { }
        }
    }

    for action in &mut stack.redo {
        match action {
            Action::Insert(ncs) => {
                replace_entity_id_(old, new, &mut ncs.1);
            },
            Action::Remove(ref mut es) => { 
                replace_entity_id2_(old, new, es);
            },
            Action::InsertConnection(ref mut con) => {
                if let Some(c_new) = replace_entity_id3_(old, new, con.clone()) {
                    *con = c_new;
                }
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
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
                        e.rotation.unwrap_or(Quat::IDENTITY),
                        e.inputs.unwrap(),
                        e.outputs.unwrap(),
                        font.clone(),
                    )
                )
            }
            NodeType::HighConst => {
                Some(Gate::high_const(
                        commands, 
                        e.position, 
                        e.rotation.unwrap_or(Quat::IDENTITY),
                        font.clone()
                ))
            }
            NodeType::LowConst => {
                Some(Gate::low_const(
                        commands, 
                        e.position, 
                        e.rotation.unwrap_or(Quat::IDENTITY),
                        font.clone()
                ))
            }
            NodeType::ToggleSwitch => {
                if let Some(NodeState::ToggleSwitch(state)) = e.state {
                    Some(ToggleSwitch::new(commands, e.position, e.rotation.unwrap_or(Quat::IDENTITY), state))
                } else { None }
            }
            NodeType::Clock => {
                if let Some(NodeState::Clock(x1, x2, x3)) = e.state {
                    Some(Clk::spawn(commands, e.position, e.rotation.unwrap_or(Quat::IDENTITY), x1, x2, x3))
                } else { None }
            }
            NodeType::LightBulb => {
                if let Some(NodeState::LightBulb(state)) = e.state {
                    Some(LightBulb::spawn(commands, e.position, e.rotation.unwrap_or(Quat::IDENTITY), state))
                } else { None }
            }
            NodeType::SevenSegmentDisplay => {
                Some(SevenSegmentDisplay::spawn(commands, e.position, e.rotation.unwrap_or(Quat::IDENTITY)))
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
) -> Option<(Vec<NodusComponent>, HashSet<(ConnInfo, ConnInfo, Entity)>)> {
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
                            if let Ok((_entity, line)) = q_line.get(connection) {
                                if let Ok(parent1) = q_parent.get(line.output.entity) {
                                    if let Ok(parent2) = q_parent.get(line.input.entity) {
                                        // By using a HashSet we dont need to check
                                        // if a connection already exists.
                                        let c = (
                                            // ConnInfo usually holds a reference to a connector
                                            // and not the gate itself, but we gonna reuse it here.
                                            ConnInfo { entity: parent1.0, index: line.output.index },
                                            ConnInfo { entity: parent2.0, index: line.input.index },
                                            connection
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

#[derive(Debug, Clone, PartialEq)]
pub struct DisconnectEventUndo {
    pub connection: Entity,
    pub in_parent: Option<Entity>,
    pub action: UndoEvent,
}

pub fn disconnect_event_system_undo(
    mut commands: Commands,
    mut ev_disconnect: EventReader<DisconnectEventUndo>,
    q_line: Query<&ConnectionLine>,
    mut q_conn: Query<(&Parent, Entity, &mut Connections)>,
    mut q_parent: Query<&mut Targets>,
    mut q_input: Query<&mut Inputs>,
    mut stack: ResMut<UndoStack>,
) {
    for ev in ev_disconnect.iter() {
        if let Some(con) = disconnect(
            &mut commands, 
            &q_line, 
            &mut q_conn, 
            &mut q_parent, 
            &mut q_input, 
            &DisconnectEvent { 
                connection: ev.connection, 
                in_parent: ev.in_parent 
            }) 
        {
            match ev.action {
                UndoEvent::Undo => {
                    stack.undo.push(Action::InsertConnection(con));
                },
                UndoEvent::Redo => {
                    stack.redo.push(Action::InsertConnection(con));
                }
            }
        }
    }
}
