use super::{
    core::{Name, *},
    graphics::{clk::*, light_bulb::*, toggle_switch::*},
    serialize::*,
    undo::*,
};
use bevy::prelude::*;
use nodus::world2d::interaction2d::{Drag, Selected};
use nodus::world2d::{InteractionMode, Lock};

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
                eprintln!("yup");
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
