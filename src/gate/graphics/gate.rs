use super::*;
use crate::gate::core::{State, *};
use crate::gate::serialize::*;
use bevy::prelude::*;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};
use lyon_tessellation::path::path::Builder;
use nodus::world2d::interaction2d::{Draggable, Interactable, Selectable};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

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
    fn body_from_path(position: Vec3, rotation: Quat, path: PathBuilder) -> ShapeBundle {
        GeometryBuilder::build_as(
            &AnsiGateShape { path: path.build() },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 6.0),
            },
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(rotation),
        )
    }

    pub fn body(position: Vec3, rotation: Quat, size: Vec2) -> ShapeBundle {
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
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(rotation),
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
        rotation: Quat,
        size: Vec2,
        in_range: NodeRange,
        out_range: NodeRange,
        ins: usize,
        outs: usize,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>,
        standard: SymbolStandard,
    ) -> Entity {
        let gate = commands
            .spawn()
            .insert(Self {
                inputs: ins as u32,
                outputs: outs as u32,
                in_range,
                out_range,
            })
            .insert(Name(name.to_string()))
            .insert(Inputs(vec![State::None; ins]))
            .insert(Outputs(vec![State::None; outs]))
            .insert(Transitions(functions))
            .insert(Targets(vec![TargetMap::from(HashMap::new()); outs]))
            .id();

        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let distances;

        match standard {
            SymbolStandard::ANSI(path) => {
                distances = get_distances(ins as f32, outs as f32, size.x, size.y);
                commands.entity(gate).insert_bundle(Gate::body_from_path(
                    Vec3::new(position.x, position.y, z),
                    rotation,
                    path,
                ));
            }
            SymbolStandard::BS(font, symbol, inverted) => {
                distances = get_distances(ins as f32, outs as f32, size.x, size.y);
                commands
                    .entity(gate)
                    .insert_bundle(Gate::body(
                        Vec3::new(position.x, position.y, z),
                        rotation,
                        Vec2::new(distances.width, distances.height),
                    ))
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
                    })
                    .id();
                commands.entity(gate).push_children(&[symbol]);

                if inverted {
                    let radius = size.y * 0.08;
                    let id = commands
                        .spawn_bundle(Gate::invert_bs(
                            Vec3::new(size.x / 2. + radius, 0., z),
                            radius,
                        ))
                        .id();

                    commands.entity(gate).push_children(&[id]);
                }
            }
        }

        commands
            .entity(gate)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(distances.width, distances.height),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true });

        let mut entvec: Vec<Entity> = Vec::new();
        for i in 1..=ins {
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
                format!("x{}", i),
            ));
        }
        for i in 1..=outs {
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
                format!("y{}", i),
            ));
        }
        commands.entity(gate).push_children(&entvec);

        gate
    }
}

impl Gate {
    pub fn not_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat,
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "NOT Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 1, max: 1 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
            trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
            },],
            SymbolStandard::BS(font, "1".to_string(), true),
        );
        commands.entity(g).insert(NodeType::Not);
        g
    }

    pub fn not_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::not_gate_bs_(commands, position, rotation, 1, 1, font)
    }

    pub fn and_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat, 
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "AND Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
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
        commands.entity(g).insert(NodeType::And);
        g
    }

    pub fn and_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::and_gate_bs_(commands, position, rotation, 2, 1, font)
    }

    pub fn nand_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat, 
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "NAND Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
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
        commands.entity(g).insert(NodeType::Nand);
        g
    }

    pub fn nand_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::nand_gate_bs_(commands, position, rotation, 2, 1, font)
    }

    pub fn or_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat, 
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "OR Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
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
        commands.entity(g).insert(NodeType::Or);
        g
    }

    pub fn or_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::or_gate_bs_(commands, position, rotation, 2, 1, font)
    }

    pub fn nor_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat, 
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "NOR Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
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
        commands.entity(g).insert(NodeType::Nor);
        g
    }

    pub fn nor_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::nor_gate_bs_(commands, position, rotation, 2, 1, font)
    }

    pub fn xor_gate_bs_(
        commands: &mut Commands,
        position: Vec2,
        rotation: Quat, 
        ins: usize,
        outs: usize,
        font: Handle<Font>,
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "XOR Gate",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_HEIGHT),
            NodeRange { min: 2, max: 16 },
            NodeRange { min: 1, max: 1 },
            ins,
            outs,
            trans![|inputs| {
                let mut ret = State::Low;
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
        commands.entity(g).insert(NodeType::Xor);
        g
    }

    pub fn xor_gate_bs(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        Self::xor_gate_bs_(commands, position, rotation, 2, 1, font)
    }

    pub fn high_const(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "HIGH Constant",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_WIDTH),
            NodeRange { min: 0, max: 0 },
            NodeRange { min: 1, max: 1 },
            0,
            1,
            trans![|_| { State::High },],
            SymbolStandard::BS(font, "1".to_string(), false),
        );
        commands.entity(g).insert(NodeType::HighConst);
        g
    }

    pub fn low_const(
        commands: &mut Commands, 
        position: Vec2, 
        rotation: Quat, 
        font: Handle<Font>
    ) -> Entity {
        let g = Gate::spawn(
            commands,
            "Low Constant",
            position,
            rotation,
            Vec2::new(GATE_WIDTH, GATE_WIDTH),
            NodeRange { min: 0, max: 0 },
            NodeRange { min: 1, max: 1 },
            0,
            1,
            trans![|_| { State::Low },],
            SymbolStandard::BS(font, "0".to_string(), false),
        );
        commands.entity(g).insert(NodeType::LowConst);
        g
    }
}

pub struct ChangeInput {
    pub gate: Entity,
    pub to: u32,
}

pub fn change_input_system(
    mut commands: Commands,
    mut ev_connect: EventReader<ChangeInput>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    mut q_gate: Query<(
        Entity,
        &mut Gate,
        &mut Inputs,
        &mut Interactable,
        &GlobalTransform,
        Option<&BritishStandard>,
    )>,
    q_connectors: Query<&Children>,
    mut q_connector: Query<(&mut Connector, &mut Transform, &Connections)>,
) {
    for ev in ev_connect.iter() {
        if let Ok((gent, mut gate, mut inputs, mut interact, transform, bs)) =
            q_gate.get_mut(ev.gate)
        {
            // Update input count
            gate.inputs = ev.to;

            let translation = transform.translation;

            // Update input vector
            inputs.resize(gate.inputs as usize, State::None);

            // If the logic component is BS it has a box as body.
            // We are going to resize it in relation to the number
            // of input connectors.
            let dists = if let Some(_) = bs {
                let dists = get_distances(
                    gate.inputs as f32,
                    gate.outputs as f32,
                    GATE_WIDTH,
                    GATE_HEIGHT,
                );

                // Update bounding box
                interact.update_size(0., 0., dists.width, dists.height);

                let gate = Gate::body(
                    Vec3::new(translation.x, translation.y, translation.z),
                    transform.rotation,
                    Vec2::new(dists.width, dists.height),
                );

                // Update body
                commands.entity(ev.gate).remove_bundle::<ShapeBundle>();
                commands.entity(ev.gate).insert_bundle(gate);

                dists
            } else {
                get_distances(
                    gate.inputs as f32,
                    gate.outputs as f32,
                    GATE_SIZE,
                    GATE_SIZE,
                )
            };

            // Update connectors attached to this gate
            let mut max = 0;
            if let Ok(connectors) = q_connectors.get(ev.gate) {
                for connector in connectors.iter() {
                    if let Ok((conn, mut trans, conns)) = q_connector.get_mut(*connector) {
                        if conn.ctype == ConnectorType::In {
                            if conn.index < ev.to as usize {
                                trans.translation = Vec3::new(
                                    -GATE_SIZE * 0.6,
                                    dists.offset + (conn.index + 1) as f32 * dists.in_step,
                                    0.,
                                );
                                if max < conn.index {
                                    max = conn.index;
                                }
                            } else {
                                // Remove connector if neccessary. This includes logical
                                // links between gates and connection line entities.
                                for &c in conns.iter() {
                                    ev_disconnect.send(DisconnectEvent {
                                        connection: c,
                                        in_parent: Some(gent),
                                    });
                                }

                                // Finally remove entity.
                                commands.entity(*connector).despawn_recursive();
                            }
                        }
                    }
                }
            }

            // If the expected amount of connectors exceeds the factual
            // amount, add new connectors to the gate.
            let mut entvec: Vec<Entity> = Vec::new();
            for i in (max + 2)..=ev.to as usize {
                entvec.push(Connector::with_line(
                    &mut commands,
                    Vec3::new(
                        -GATE_SIZE * 0.6,
                        dists.offset + i as f32 * dists.in_step,
                        translation.z,
                    ),
                    GATE_SIZE * 0.1,
                    ConnectorType::In,
                    i - 1,
                    format!("y{}", i),
                ));
            }
            if !entvec.is_empty() {
                commands.entity(ev.gate).push_children(&entvec);
            }
        }
    }
}
