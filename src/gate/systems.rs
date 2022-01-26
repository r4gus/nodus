use bevy::prelude::*;
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable, Drag, Selected};
use nodus::world2d::{InteractionMode, Lock};
use super::{
    core::*,
    graphics::{
        light_bulb::*,
        toggle_switch::*,
        gate::*,
        clk::*,
    },
};

pub fn shortcut_system(
    mut mode: ResMut<InteractionMode>,
    input_keyboard: Res<Input<KeyCode>>,
    lock: Res<Lock>,
) {
    if lock.0 { return; }

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
) {
    if input_keyboard.pressed(KeyCode::Delete) {
        // Iterate over every selected gate and its children.
        for (entity, children) in q_gate.iter() {
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

/*
pub fn select_multiple_system(
    mb: Res<Input<MouseButton>>,
    q_gate: Query<
        (Entity, &Children),
        (
            With<Selected>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>, With<Clk>)>,
        ),
    >,
) {

}
*/
