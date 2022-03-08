use super::*;
use crate::gate::core::{State, *};
use crate::gate::serialize::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use lyon_tessellation::path::path::Builder;
use nodus::world2d::interaction2d::{Draggable, Hover, Interactable, Selectable};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, PartialEq, Hash, Component)]
pub struct Segment {
    nr: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct SegmentShape {
    size: f32,
}

#[derive(Debug, Clone, PartialEq, Hash, Component)]
pub struct SevenSegmentDisplay {
    segments: Vec<Entity>,
}

impl Geometry for SegmentShape {
    fn add_geometry(&self, b: &mut Builder) {
        let mut path = PathBuilder::new();
        let step = self.size / 5.0;
    
        path.move_to(Vec2::new(0.0, 0.0));
        path.line_to(Vec2::new(step, step));
        path.line_to(Vec2::new(path.current_position().x + 3.0 * step, path.current_position().y));
        path.line_to(Vec2::new(path.current_position().x + step, path.current_position().y - step));
        path.line_to(Vec2::new(path.current_position().x - step, path.current_position().y - step));
        path.line_to(Vec2::new(path.current_position().x - 3.0 * step, path.current_position().y));
        path.close();
        b.concatenate(&[path.build().0.as_slice()]);
    }
}

impl SegmentShape {
    pub fn spawn(commands: &mut Commands, position: Vec3, rotation: Quat, size: f32, nr: u8) -> Entity {
        let segment = GeometryBuilder::build_as(
            &SegmentShape {
                size,
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 2.0),
            },
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(rotation),
        );

        commands
            .spawn_bundle(segment)
            .insert(Segment { nr })
            .id()
    }
}

impl SevenSegmentDisplay {
    pub fn spawn(commands: &mut Commands, position: Vec2, rotation: Quat) -> Entity {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let segment_size = GATE_WIDTH;
        let x = segment_size * 0.5;
        let y = segment_size * 1.2;
        let coords = vec![
            (Vec3::new(-x, y, 0.1), Quat::IDENTITY),
            (Vec3::new(segment_size + 2.0 - x, y - 2.0, 0.1), Quat::from_rotation_z(-std::f32::consts::PI/ 2.0)),
            (Vec3::new(segment_size + 2.0 - x, y - segment_size - 6.0, 0.1), Quat::from_rotation_z(-std::f32::consts::PI/ 2.0)),
            (Vec3::new(-x, y - segment_size * 2.0 - 8.0, 0.1), Quat::IDENTITY),
            (Vec3::new(-2.0 - x, y - segment_size - 6.0, 0.1), Quat::from_rotation_z(-std::f32::consts::PI/ 2.0)),
            (Vec3::new(-2.0 - x, y - 2.0, 0.1), Quat::from_rotation_z(-std::f32::consts::PI/ 2.0)),
            (Vec3::new(-x, y - segment_size - 4.0, 0.1), Quat::IDENTITY),
        ];
        
        let mut segments: Vec<Entity> = Vec::new();
        
        for (nr, (pos, rot)) in coords.iter().enumerate() {
            segments.push(
                SegmentShape::spawn(
                    commands, 
                    *pos, 
                    *rot,
                    segment_size,
                    nr as u8
                )
            );
        }
        
        let parent = commands
            .spawn_bundle(
                Gate::body(
                    Vec3::new(position.x, position.y, z),
                    rotation,
                    Vec2::new(GATE_WIDTH * 2.0, GATE_HEIGHT * 2.0),
                )
            )
            .insert(
                Transform::from_xyz(position.x, position.y, z)
                    .with_rotation(rotation)
            )
            .insert(
                GlobalTransform::from_xyz(position.x, position.y, z)
                    .with_rotation(rotation)
            )
            .insert(SevenSegmentDisplay { segments: segments.clone() })
            .insert(Name("7-Segment Display".to_string()))
            .insert(Inputs(vec![State::None; 4]))
            .insert(NodeType::SevenSegmentDisplay)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_WIDTH * 2.0, GATE_HEIGHT * 2.0),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        for i in 0..4 {
            segments.push(
                Connector::with_line_vert(
                    commands,
                    Vec3::new(
                        -GATE_WIDTH * 2.0 * 0.375 + i as f32 * GATE_WIDTH * 2.0 * 0.25, 
                        -GATE_HEIGHT * 2.0 * 0.7, 
                        0.
                    ),
                    GATE_SIZE * 0.1,
                    ConnectorType::In,
                    i,
                    format!("x{}", i),
                )
            );
        }

        commands
            .entity(parent)
            .push_children(&segments);

        parent
    }
}

const COLOR_OFF: Color = Color::WHITE;
const COLOR_ON: Color = Color::RED;

const DISPLAY_COLORS: [[Color; 7]; 16] = [
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF],
    [COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_OFF, COLOR_OFF],
    [COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON],
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_ON],
    [COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_OFF, COLOR_OFF],
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON],
    [COLOR_OFF, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF],
    [COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_OFF, COLOR_ON],
    [COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON, COLOR_ON],
    [COLOR_ON, COLOR_OFF, COLOR_OFF, COLOR_OFF, COLOR_ON, COLOR_ON, COLOR_ON],
];

pub fn segment_system(
    q_seg: Query<(&Inputs, &SevenSegmentDisplay)>,
    mut draw: Query<&mut DrawMode>,
) {
    for (inputs, display) in q_seg.iter() {
        // Inputs are treated as little endian, i.e. 2^3 + 2^2 + 2^1 + 2^0.
        let mut i = if inputs[3] == State::High { 1 } else { 0 };
        i |= if inputs[2] == State::High { 1 } else { 0 } << 1;
        i |= if inputs[1] == State::High { 1 } else { 0 } << 2;
        i |= if inputs[0] == State::High { 1 } else { 0 } << 3;

        for j in 0..7 {
            let e = display.segments[j];
            
            if let Ok(mut mode) = draw.get_mut(e) {
                if let DrawMode::Outlined {
                    ref mut fill_mode,
                    outline_mode: _,
                } = *mode 
                {
                    fill_mode.color = DISPLAY_COLORS[i][j];
                }
            }
        }
    }
}
