pub mod light_bulb;
pub mod toggle_switch;
pub mod clk;
pub mod gate;
pub mod connector;
pub mod connection_line;
pub mod background;

use core::sync::atomic::AtomicI32;
use nodus::world2d::interaction2d::Selected;
use crate::gate::core::Connector;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub static Z_INDEX: AtomicI32 = AtomicI32::new(10);

pub const GATE_SIZE: f32 = 128.;
pub const GATE_WIDTH: f32 = 64.;
pub const GATE_HEIGHT: f32 = 128.;

/// Marker component for entities that act as highlighters.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct Highlighter;

/// Marker component for entities that are highlighted.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct Highlighted;

impl Highlighter {
    /// Spawn a new highlight entity that uses the given path for its shape.
    pub fn spawn(commands: &mut Commands, path: &Path) -> Entity {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &path.0,
                DrawMode::Fill(FillMode::color(Color::rgba(0.62, 0.79, 0.94, 0.5))),
                Transform::from_xyz(0.0, 0.0, 0.1),
            )).insert(Highlighter).id()
    }
}

/// Hightlight a entity (gate, input control, ...) the user has clicked on.
pub fn highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Path), (Added<Selected>, Without<Highlighted>, Without<Connector>)>,
) {
    for (entity, path) in query.iter() {
        let h = Highlighter::spawn(&mut commands, &path);
        commands.entity(entity).add_child(h);
        commands.entity(entity).insert(Highlighted);
    }
}

/// Remove highlighting as soon as the entity isn't selected anymore.
pub fn remove_highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Children), (With<Highlighted>, Without<Selected>)>,
    q_child: Query<Entity, With<Highlighter>>,
) {
    for (parent, children) in query.iter() {
        commands.entity(parent).remove::<Highlighted>();

        for &child in children.iter() {
            if let Ok(entity) = q_child.get(child) {
                commands.entity(entity).despawn_recursive(); 
            }
        }
    }
}

/// Redraw the highlight of a highlighted entity if the path of its main shape has changed.
pub fn change_highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Children, &Path), (Changed<Path>, With<Highlighted>)>,
    q_child: Query<Entity, With<Highlighter>>,
) {
    for (parent, children, path) in query.iter() {
        for &child in children.iter() {
            if let Ok(entity) = q_child.get(child) {
                commands.entity(entity).despawn_recursive(); 
            }
        }

        let h = Highlighter::spawn(&mut commands, &path);
        commands.entity(parent).add_child(h);
    }
}
