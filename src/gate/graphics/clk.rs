use crate::gate::core::{*, State};
use super::*;
use crate::gate::serialize::*;
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable};
use std::sync::atomic::Ordering;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;
use lyon_tessellation::path::path::Builder;

/// Clock (clk) marker component.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct Clk(pub f32, pub f32);

impl Clk {
    pub fn spawn(
        commands: &mut Commands,
        position: Vec2,
        clk: f32,
        start: f32,
        state: State,
    ) -> Entity {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let shape = shapes::Rectangle {
            extents: Vec2::new(GATE_SIZE, GATE_SIZE),
            ..shapes::Rectangle::default()
        };
        
        let clk = commands
            .spawn_bundle(
                GeometryBuilder::build_as(
                    &shape,
                     DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::WHITE),
                        outline_mode: StrokeMode::new(Color::BLACK, 6.0),
                    },
                    Transform::from_xyz(position.x, position.y, z),
                ))
            .insert(Clk(clk, start))
            .insert(Name("Clock".to_string()))
            .insert(NodeType::Clock)
            .insert(Outputs(vec![state]))
            .insert(Targets(vec![TargetMap::from(HashMap::new())]))
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE, GATE_SIZE),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .with_children(|parent| {
                parent.spawn_bundle(
                    GeometryBuilder::build_as(
                        &ClkShape { size: GATE_SIZE / 2. },
                        DrawMode::Stroke(StrokeMode::new(if state == State::High { Color::BLUE } else { Color::BLACK }, 16.0)),
                        Transform::from_xyz(0., 0., 1.),
                    )
                );
            }).id();

        let conn = Connector::with_line(
            commands,
            Vec3::new(GATE_SIZE * 0.75, 0., 0.),
            GATE_SIZE * 0.1,
            ConnectorType::Out,
            0,
        );

        commands.entity(clk).add_child(conn);
        clk
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ClkShape {
    size: f32,
}

impl Geometry for ClkShape {
    fn add_geometry(&self, b: &mut Builder) {
        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(- self.size * 0.75, self.size / 2.));
        path.line_to(Vec2::new(0., self.size / 2.));
        path.line_to(Vec2::new(0., - self.size / 2.));
        path.line_to(Vec2::new(self.size * 0.75, - self.size / 2.));
        b.concatenate(&[path.build().0.as_slice()]);
    }
}

pub fn clk_system(
    mut commands: Commands,
    mut q_clk: Query<(&Children, &mut Clk, &mut Outputs)>,
    mut draw: Query<&mut DrawMode, Without<Connector>>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();

    for (children, mut clk, mut outs) in q_clk.iter_mut() {
        clk.1 += delta;

        if clk.1 >= clk.0 { 
            clk.1 = 0.0; 
            outs[0] = match outs[0] {
                State::High => State::Low,
                _ => State::High,
            };

            for &child in children.iter() {
                if let Ok(mut mode) = draw.get_mut(child) {
                    if let DrawMode::Stroke(ref mut stroke_mode) = *mode {
                        stroke_mode.color = match outs[0] {
                            State::High => Color::BLUE,
                            _ => Color::BLACK,
                        };
                    }
                }
            }
        }
    }
}
