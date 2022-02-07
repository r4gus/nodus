use crate::gate::core::{*, State};
use crate::gate::serialize::*;
use super::*;
use nodus::world2d::interaction2d::{Interactable, Hover, Selectable, Draggable};
use std::sync::atomic::Ordering;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;
use lyon_tessellation::path::path::Builder;

/// A toggle switch that can be either on or off.
///
/// If on, it will propagate a State::High signal to all connected
/// logical components, else State::Low;
#[derive(Debug, Clone, PartialEq, Hash, Component)]
pub struct ToggleSwitch;

impl ToggleSwitch {
    /// Crate a new toggle switch at the specified position.
    pub fn new(
        commands: &mut Commands,
        position: Vec2,
        state: State,
    ) -> Entity {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let switch = GeometryBuilder::build_as(
            &ToggleSwitchShape { size: GATE_SIZE / 4. },
             DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 8.0),
            },
            Transform::from_xyz(position.x, position.y, z),
        );

        let parent = commands
            .spawn_bundle(switch)
            .insert(ToggleSwitch)
            .insert(Name("Toggle Switch".to_string()))
            .insert(Inputs(vec![state]))
            .insert(Outputs(vec![state]))
            .insert(Transitions(trans![|inputs| inputs[0]]))
            .insert(Targets(vec![TargetMap::from(HashMap::new())]))
            .insert(NodeType::ToggleSwitch)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE, GATE_SIZE),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        let child = Connector::with_line(
            commands,
            Vec3::new(GATE_SIZE * 0.75, 0., 0.),
            GATE_SIZE * 0.1,
            ConnectorType::Out,
            0,
        );
        
        let factor = if state == State::High { 1. } else { -1. };

        let nod = GeometryBuilder::build_as(
            &shapes::Circle {
                radius: GATE_SIZE / 4.,
                center: Vec2::new(0., 0.),
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 8.0),
            },
            Transform::from_xyz(factor * GATE_SIZE / 4., 0., 1.),
        );

        let nod_child = commands
            .spawn_bundle(nod)
            .insert(Switch)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE / 2., GATE_SIZE / 2.),
                1,
            ))
            .id();

        commands
            .entity(parent)
            .push_children(&vec![child, nod_child]);

        parent
    }
}

/// Switch represents the part of the toggle switch the user can click on.
#[derive(Debug, Clone, PartialEq, Hash, Component)]
pub struct Switch;

/// Defines the basic shape of the toggle switch by implementing Geometry.
#[derive(Debug, Clone, PartialEq)]
struct ToggleSwitchShape {
    size: f32,
}

impl Geometry for ToggleSwitchShape {
    fn add_geometry(&self, b: &mut Builder) {
        let mut path = PathBuilder::new();

        path.move_to(Vec2::new(-self.size, -self.size));
        path.arc(
            Vec2::new(-self.size, 0.),
            Vec2::new(self.size, self.size),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(self.size, self.size));
        path.arc(
            Vec2::new(self.size, 0.),
            Vec2::new(self.size, self.size),
            -std::f32::consts::PI,
            0.,
        );
        path.close();
        b.concatenate(&[path.build().0.as_slice()]);
    }
}

/// Register clicks on a switch and change its state accordingly.
pub fn toggle_switch_system(
    mut commands: Commands,
    mut q_outputs: Query<&mut Inputs>,
    mut q_switch: Query<(&Parent, &mut Transform), (With<Hover>, With<Switch>)>,
    mb: Res<Input<MouseButton>>,
) {
    if mb.just_pressed(MouseButton::Left) {
        for (parent, mut transform) in q_switch.iter_mut() {
            if let Ok(mut inputs) = q_outputs.get_mut(parent.0) {
                let next = match inputs[0] {
                    State::High => {
                        transform.translation.x -= GATE_SIZE / 2.;
                        State::Low
                    }
                    _ => {
                        transform.translation.x += GATE_SIZE / 2.;
                        State::High
                    }
                };
                inputs[0] = next;
            }
        }
    }
}
