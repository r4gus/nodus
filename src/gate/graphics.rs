pub mod light_bulb;
pub mod toggle_switch;
pub mod clk;
pub mod gate;
pub mod connector;
pub mod connection_line;
pub mod background;

use core::sync::atomic::AtomicI32;
use nodus::world2d::interaction2d::Selected;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub static Z_INDEX: AtomicI32 = AtomicI32::new(10);

pub const GATE_SIZE: f32 = 128.;
pub const GATE_WIDTH: f32 = 64.;
pub const GATE_HEIGHT: f32 = 128.;

#[derive(Debug, Clone, PartialEq, Component)]
pub struct Highlighter;

#[derive(Debug, Clone, PartialEq, Component)]
pub struct Highlighted;

pub fn highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Path), (Added<Selected>, Without<Highlighted>)>,
) {
    for (entity, path) in query.iter() {
        eprintln!("add");
        let h = commands
            .spawn_bundle(GeometryBuilder::build_as(
                &path.0,
                DrawMode::Fill(FillMode::color(Color::rgba(0.62, 0.79, 0.94, 0.5))),
                Transform::from_scale(Vec3::new(1.3, 1.2, 1.0)).with_translation(Vec3::new(0.0, 0.0, -2.0)),
            )).insert(Highlighter).id();
        commands.entity(entity).add_child(h);
        commands.entity(entity).insert(Highlighted);
    }
}

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
