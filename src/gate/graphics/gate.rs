use crate::gate::core::{*, State};
use super::*;
use bevy::prelude::*;
use bevy_prototype_lyon::{
    prelude::*,
    entity::ShapeBundle,
};
use lyon_tessellation::path::path::Builder;
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable};
use std::sync::atomic::Ordering;
use std::collections::HashMap;

pub struct GateSize {
    pub width: f32,
    pub height: f32,
    pub in_step: f32,
    pub out_step: f32,
    pub offset: f32,
}

pub fn get_distances(cin: f32, cout: f32, width: f32, _height: f32) -> GateSize {
    let factor = if cin >= cout { cin } else { cout };
    let height = _height
        + if factor > 2. {
            (factor - 1.) * _height / 2.
        } else {
            0.
        };
    let in_step = -(height / (cin + 1.));
    let out_step = -(height / (cout + 1.));
    let offset = height / 2.;

    GateSize {
        width,
        height,
        in_step,
        out_step,
        offset,
    }
}

pub enum SymbolStandard {
    ANSI(PathBuilder),
    // Font | Symbol | inverted?
    BS(Handle<Font>, String, bool), // British System 3939
}

pub struct AnsiGateShape {
    pub path: Path,
}

impl Geometry for AnsiGateShape {
    fn add_geometry(&self, b: &mut Builder) {
        b.concatenate(&[self.path.0.as_slice()]);
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct BritishStandard;

impl Gate {
    fn body_from_path(position: Vec3, path: PathBuilder) -> ShapeBundle {
        GeometryBuilder::build_as(
            &AnsiGateShape { path: path.build() },
             DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 6.0),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        )
    }

    pub fn body(position: Vec3, size: Vec2) -> ShapeBundle {
        let shape = shapes::Rectangle {
            extents: Vec2::new(size.x, size.y),
            ..shapes::Rectangle::default()
        };

        GeometryBuilder::build_as(
            &shape,
             DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 6.0),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        )
    }

    fn invert_bs(position: Vec3, radius: f32) -> ShapeBundle {
        let shape = shapes::Circle {
            radius,
            ..shapes::Circle::default()
        };

        GeometryBuilder::build_as(
            &shape,
             DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 6.0),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        )
    }

    /// Spawn a new gate at the specified position in the world.
    pub fn spawn(
        commands: &mut Commands,
        name: &str,
        position: Vec2,
        size: Vec2,
        in_range: NodeRange,
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>,
        standard: SymbolStandard,
    ) {
        let gate = commands
            .spawn()
            .insert(Self {
                inputs: in_range.min,
                outputs: out_range.min,
                in_range,
                out_range,
            })
            .insert(Name(name.to_string()))
            .insert(Inputs(vec![State::None; in_range.min as usize]))
            .insert(Outputs(vec![State::None; out_range.min as usize]))
            .insert(Transitions(functions))
            .insert(Targets(vec![TargetMap::from(HashMap::new()); out_range.min as usize]))
            .id();

        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let distances;

        match standard {
            SymbolStandard::ANSI(path) => {
                distances = get_distances(in_range.min as f32, out_range.min as f32, size.x, size.y);
                commands.entity(gate).insert_bundle(Gate::body_from_path(Vec3::new(position.x, position.y, z), path));
            },
            SymbolStandard::BS(font, symbol, inverted) => {
                distances = get_distances(in_range.min as f32, out_range.min as f32, size.x, size.y);
                commands.entity(gate)
                    .insert_bundle(Gate::body(Vec3::new(position.x, position.y, z), Vec2::new(size.x, size.y)))
                    .insert(BritishStandard);

                let symbol = commands
                        .spawn_bundle(Text2dBundle {
                            text: Text::with_section(
                                &symbol,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::BLACK,
                                },
                                TextAlignment {
                                    horizontal: HorizontalAlign::Center,
                                    ..Default::default()
                                },
                            ),
                            transform: Transform::from_xyz(0., 0., z),
                            ..Default::default()
                        }).id();
                commands.entity(gate).push_children(&[symbol]);

                if inverted {
                    let radius = size.y * 0.08;
                    let id = commands.spawn_bundle(Gate::invert_bs(Vec3::new(size.x / 2. + radius, 0., z), radius)).id();

                    commands.entity(gate).push_children(&[id]);
                }
            }
        }

        commands.entity(gate)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(distances.width, distances.height),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true });

        let mut entvec: Vec<Entity> = Vec::new();
        for i in 1..=in_range.min {
            entvec.push(Connector::with_line(
                commands,
                Vec3::new(
                    -size.y * 0.6,
                    distances.offset + i as f32 * distances.in_step,
                    z,
                ),
                size.y * 0.1,
                ConnectorType::In,
                (i - 1) as usize,
            ));
        }
        for i in 1..=out_range.min {
            entvec.push(Connector::with_line(
                commands,
                Vec3::new(
                    size.y * 0.6,
                    distances.offset + i as f32 * distances.out_step,
                    z,
                ),
                size.y * 0.1,
                ConnectorType::Out,
                (i - 1) as usize,
            ));
        }
        commands.entity(gate).push_children(&entvec);
    }
}

impl Gate {
    pub fn not_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "NOT Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 1, max: 1 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
            },],
            SymbolStandard::BS(font, "1".to_string(), true),
        );
    }

    pub fn and_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "AND Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                let mut ret = State::High;
                for i in inputs {
                    match i {
                        State::None => {
                            ret = State::None;
                        }
                        State::Low => {
                            ret = State::Low;
                            break;
                        }
                        State::High => {}
                    }
                }
                ret
            },],
            SymbolStandard::BS(font, "&".to_string(), false),
        );
    }

    pub fn nand_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "NAND Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                let mut ret = State::Low;
                for i in inputs {
                    match i {
                        State::None => {
                            ret = State::None;
                        }
                        State::Low => {
                            ret = State::High;
                            break;
                        }
                        State::High => {}
                    }
                }
                ret
            },],
            SymbolStandard::BS(font, "&".to_string(), true),
        );
    }

    pub fn or_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "OR Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                let mut ret = State::Low;
                for i in inputs {
                    match i {
                        State::None => {
                            ret = State::None;
                        }
                        State::Low => {}
                        State::High => {
                            ret = State::High;
                            break;
                        }
                    }
                }
                ret
            },],
            SymbolStandard::BS(font, "≥1".to_string(), false),
        );
    }

    pub fn nor_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "NOR Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                let mut ret = State::High;
                for i in inputs {
                    match i {
                        State::None => {
                            ret = State::None;
                        }
                        State::Low => {}
                        State::High => {
                            ret = State::Low;
                            break;
                        }
                    }
                }
                ret
            },],
            SymbolStandard::BS(font, "≥1".to_string(), true),
        );
    }

    pub fn xor_gate_bs(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "XOR Gate",
            position,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                let mut ret = State::None;
                for i in inputs {
                    match i {
                        State::None => {}
                        State::Low => {}
                        State::High => match ret {
                            State::None => {
                                ret = State::High;
                            }
                            State::Low => {
                                ret = State::High;
                            }
                            State::High => {
                                ret = State::Low;
                            }
                        },
                    }
                }
                ret
            },],
            SymbolStandard::BS(font, "=1".to_string(), false),
        );
    }

    pub fn high_const(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "HIGH Constant",
            position,
            Vec2::new(GATE_WIDTH, GATE_WIDTH),
            NodeRange { min: 0, max: 0 },
            NodeRange { min: 1, max: 1 },
            trans![|_| { State::High },],
            SymbolStandard::BS(font, "1".to_string(), false),
        );
    }

    pub fn low_const(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::spawn(
            commands,
            "Low Constant",
            position,
            Vec2::new(GATE_WIDTH, GATE_WIDTH),
            NodeRange { min: 0, max: 0 },
            NodeRange { min: 1, max: 1 },
            trans![|_| { State::Low },],
            SymbolStandard::BS(font, "0".to_string(), false),
        );
    }
}
