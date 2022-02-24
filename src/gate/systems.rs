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
        Entity,
        (
            With<Selected>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>, With<Clk>)>,
        ),
    >,
    children: Query<&Children>,
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
    q_line: Query<(Entity, &ConnectionLine)>,
    q_parent: Query<&Parent>,
) {
    if input_keyboard.pressed(KeyCode::Delete) {
        if let Some(ncs) = crate::gate::undo::remove(
            &mut commands, 
            q_gate.iter().map(|e| e).collect(), 
            &q_node, 
            &children, 
            &q_connectors, 
            &q_line,
            &q_parent,
            &mut ev_disconnect
        ) {
            stack.undo.push(Action::Insert(ncs));
            stack.redo.clear();
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
                Some(Gate::and_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::Nand => {
                Some(Gate::nand_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::Or => {
                Some(Gate::or_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::Nor => {
                Some(Gate::nor_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::Xor => {
                Some(Gate::xor_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::Xnor => {
                None
            },
            NodeType::Not => {
                Some(Gate::not_gate_bs(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::HighConst => {
                Some(Gate::high_const(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::LowConst => {
                Some(Gate::low_const(&mut commands, ev.position, Quat::IDENTITY, font.main.clone()))
            },
            NodeType::ToggleSwitch => {
                Some(ToggleSwitch::new(&mut commands, ev.position, Quat::IDENTITY, State::Low))
            },
            NodeType::Clock => {
                Some(Clk::spawn(&mut commands, ev.position, Quat::IDENTITY, 1.0, 0.0, State::Low))
            },
            NodeType::LightBulb => {
                Some(LightBulb::spawn(&mut commands, ev.position, Quat::IDENTITY, State::None))
            },
        };

        if let Some(entity) = entity {
            stack.undo.push(Action::Remove(vec![entity]));
            stack.redo.clear();
        }
    }
}
