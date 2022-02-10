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
            .insert_resource(UndoStack {
                undo: Vec::new(),
                redo: Vec::new(),
            })
            .add_system(handle_undo_event_system)
            // The removal of components is applied at the end of a stage.
            // The system that checks for the removal must run in a later stage.
            .add_system_to_stage(CoreStage::PostUpdate, detect_removal_system);
    }
}

pub enum UndoEvent {
    Undo,
    Redo,
}

#[derive(Debug)]
pub enum Action {
    Insert(NodusComponent),
    Remove(Entity),
}

#[derive(Debug)]
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
) {
    let font: Handle<Font> = server.load("fonts/hack.bold.ttf");

    for ev in ev_undo.iter() {
        match ev {
            UndoEvent::Undo => {
                if let Some(action) = stack.undo.pop() {
                    match action {
                        Action::Insert(e) => {
                            match e.ntype {
                                NodeType::And => {
                                    let _id = Gate::and_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );

                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::Nand => {
                                    let _id = Gate::nand_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::Or => {
                                    let _id = Gate::or_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::Nor => {
                                    let _id = Gate::nor_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::Xor => {
                                    let _id = Gate::xor_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::Xnor => {}
                                NodeType::Not => {
                                    let _id = Gate::not_gate_bs_(
                                        &mut commands,
                                        e.position,
                                        e.inputs.unwrap(),
                                        e.outputs.unwrap(),
                                        font.clone(),
                                    );
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::HighConst => {
                                    let _id =
                                        Gate::high_const(&mut commands, e.position, font.clone());
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::LowConst => {
                                    let _id =
                                        Gate::low_const(&mut commands, e.position, font.clone());
                                    stack.redo.push(Action::Remove(_id));
                                }
                                NodeType::ToggleSwitch => {
                                    if let Some(NodeState::ToggleSwitch(state)) = e.state {
                                        let _id =
                                            ToggleSwitch::new(&mut commands, e.position, state);
                                        stack.redo.push(Action::Remove(_id));
                                    }
                                }
                                NodeType::Clock => {
                                    if let Some(NodeState::Clock(x1, x2, x3)) = e.state {
                                        let _id = Clk::spawn(&mut commands, e.position, x1, x2, x3);
                                        stack.redo.push(Action::Remove(_id));
                                    }
                                }
                                NodeType::LightBulb => {
                                    if let Some(NodeState::LightBulb(state)) = e.state {
                                        let _id =
                                            LightBulb::spawn(&mut commands, e.position, state);
                                        stack.redo.push(Action::Remove(_id));
                                    }
                                }
                            }
                        }
                        Action::Remove(e) => {
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
                                    ntype: nt.clone(),
                                    state: state,
                                };

                                stack.redo.push(Action::Insert(nc));
                                commands.entity(e).despawn_recursive();
                            }
                        }
                    }
                }
            }
            UndoEvent::Redo => {
                if let Some(action) = stack.redo.pop() {
                    match action {
                        Action::Insert(e) => match e.ntype {
                            NodeType::And => {
                                let _id = Gate::and_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::Nand => {
                                let _id = Gate::nand_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::Or => {
                                let _id = Gate::or_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::Nor => {
                                let _id = Gate::nor_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::Xor => {
                                let _id = Gate::xor_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::Xnor => {}
                            NodeType::Not => {
                                let _id = Gate::not_gate_bs_(
                                    &mut commands,
                                    e.position,
                                    e.inputs.unwrap(),
                                    e.outputs.unwrap(),
                                    font.clone(),
                                );
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::HighConst => {
                                let _id = Gate::high_const(&mut commands, e.position, font.clone());
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::LowConst => {
                                let _id = Gate::low_const(&mut commands, e.position, font.clone());
                                stack.undo.push(Action::Remove(_id));
                            }
                            NodeType::ToggleSwitch => {
                                if let Some(NodeState::ToggleSwitch(state)) = e.state {
                                    let _id = ToggleSwitch::new(&mut commands, e.position, state);
                                    stack.undo.push(Action::Remove(_id));
                                }
                            }
                            NodeType::Clock => {
                                if let Some(NodeState::Clock(x1, x2, x3)) = e.state {
                                    let _id = Clk::spawn(&mut commands, e.position, x1, x2, x3);
                                    stack.undo.push(Action::Remove(_id));
                                }
                            }
                            NodeType::LightBulb => {
                                if let Some(NodeState::LightBulb(state)) = e.state {
                                    let _id = LightBulb::spawn(&mut commands, e.position, state);
                                    stack.undo.push(Action::Remove(_id));
                                }
                            }
                        },
                        Action::Remove(_e) => {}
                    }
                }
            }
        }
        eprintln!("undo: {}", stack.undo.len());
        eprintln!("redo: {}", stack.redo.len());
    }
}

pub fn detect_removal_system(
    _stack: ResMut<UndoStack>,
    _removals: RemovedComponents<NodeType>,
    _q_node: Query<(
        Entity,
        &Name,
        Option<&Inputs>,
        Option<&Outputs>,
        Option<&Targets>,
        Option<&Clk>,
        &Transform,
        &NodeType,
    )>,
) {
}
