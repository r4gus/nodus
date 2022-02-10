use super::{
    core::{Name, *},
    graphics::{clk::*, light_bulb::*, toggle_switch::*},
    serialize::*,
    undo::*,
};
use bevy::prelude::*;
use nodus::world2d::interaction2d::{Drag, Selected};
use nodus::world2d::{InteractionMode, Lock};
use crate::FontAssets;

pub fn shortcut_system(
    mut mode: ResMut<InteractionMode>,
    input_keyboard: Res<Input<KeyCode>>,
    lock: Res<Lock>,
) {
    if lock.0 {
        return;
    }

    if input_keyboard.pressed(KeyCode::P) {
        *mode = InteractionMode::Pan;
    } else if input_keyboard.pressed(KeyCode::S) {
        *mode = InteractionMode::Select;
    }
}

/// Removes the drag state from draggable components.
pub fn drag_gate_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    q_dragged: Query<
        Entity,
        (
            With<Drag>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>, With<Clk>)>,
        ),
    >,
) {
    if mb.just_released(MouseButton::Left) {
        for dragged_gate in q_dragged.iter() {
            commands.entity(dragged_gate).remove::<Drag>();
        }
    }
}

/// Delete a selected gate, input control or output control.
pub fn delete_gate_system(
    mut commands: Commands,
    input_keyboard: Res<Input<KeyCode>>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    q_gate: Query<
        (Entity, &Children),
        (
            With<Selected>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>, With<Clk>)>,
        ),
    >,
    q_connectors: Query<&Connections>,
    mut stack: ResMut<UndoStack>,
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
    if input_keyboard.pressed(KeyCode::Delete) {
        // Iterate over every selected gate and its children.
        //let mut vundo = Vec::new();
        for (entity, children) in q_gate.iter() {
            // ----------------------------------- undo
            if let Ok((e, n, ip, op, t, clk, tr, nt)) = q_node.get(entity) {
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
                    NodeType::ToggleSwitch => Some(NodeState::ToggleSwitch(op.unwrap()[0])),
                    NodeType::Clock => {
                        let clk = clk.unwrap();
                        Some(NodeState::Clock(clk.0, clk.1, op.unwrap()[0]))
                    }
                    NodeType::LightBulb => Some(NodeState::LightBulb(ip.unwrap()[0])),
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

                stack.undo.push(Action::Insert(nc));
            }
            // ----------------------------------- undo

            // Get the connections for each child
            // and disconnect all.
            for &child in children.iter() {
                if let Ok(conns) = q_connectors.get(child) {
                    for &connection in conns.iter() {
                        ev_disconnect.send(DisconnectEvent {
                            connection,
                            in_parent: Some(entity),
                        });
                    }
                }
            }

            // Delete the gate itself
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertGateEvent {
    gate_type: NodeType,
    pub position: Vec2,
}

impl InsertGateEvent {
    pub fn and(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::And,
            position,
        }
    }

    pub fn nand(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Nand,
            position,
        }
    }

    pub fn or(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Or,
            position,
        }
    }
    
    pub fn nor(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Nor,
            position,
        }
    }

    pub fn not(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Not,
            position,
        }
    }

    pub fn xor(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Xor,
            position,
        }
    }

    pub fn high(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::HighConst,
            position,
        }
    }

    pub fn low(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::LowConst,
            position,
        }
    }

    pub fn toggle(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::ToggleSwitch,
            position,
        }
    }

    pub fn clk(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::Clock,
            position,
        }
    }

    pub fn light(position: Vec2) -> Self {
        Self {
            gate_type: NodeType::LightBulb,
            position,
        }
    }
}

pub fn insert_gate_system(
    mut commands: Commands,
    mut ev_insert: EventReader<InsertGateEvent>,
    mut stack: ResMut<UndoStack>,
    font: Res<FontAssets>,
) {
    use crate::gate::core::State;
    
    for ev in ev_insert.iter() {
        let entity = match ev.gate_type {
            NodeType::And => {
                Some(Gate::and_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::Nand => {
                Some(Gate::nand_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::Or => {
                Some(Gate::or_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::Nor => {
                Some(Gate::nor_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::Xor => {
                Some(Gate::xor_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::Xnor => {
                None
            },
            NodeType::Not => {
                Some(Gate::not_gate_bs(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::HighConst => {
                Some(Gate::high_const(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::LowConst => {
                Some(Gate::low_const(&mut commands, ev.position, font.main.clone()))
            },
            NodeType::ToggleSwitch => {
                Some(ToggleSwitch::new(&mut commands, ev.position, State::Low))
            },
            NodeType::Clock => {
                Some(Clk::spawn(&mut commands, ev.position, 1.0, 0.0, State::Low))
            },
            NodeType::LightBulb => {
                Some(LightBulb::spawn(&mut commands, ev.position, State::None))
            },
            _ => { None }
        };

        if let Some(entity) = entity {
            stack.undo.push(Action::Remove(entity));
        }
    }
}
