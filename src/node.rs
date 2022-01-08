use crate::radial_menu::{OpenMenuEvent, PropagateSelectionEvent, UpdateCursorPositionEvent};
use crate::{FontAssets, GameState};
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_egui::{egui, EguiContext};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::SvgPathShape;
use nodus::world2d::camera2d::MouseWorldPos;
use nodus::world2d::interaction2d::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

pub struct NodeInGamePlugin;

const NODE_GROUP: u32 = 1;
const CONNECTOR_GROUP: u32 = 2;

const LIGHT_BULB_PATH: &str = "M290.222,0C180.731,0,91.8,88.931,91.8,198.422c0,54.506,17.69,86.062,33.469,113.315l0,0
              c12.909,22.472,22.95,40.163,26.775,74.588c0.478,7.649,2.391,14.821,5.259,21.993c-2.391,6.694-3.825,13.866-3.825,21.038
              c0,8.128,1.434,15.778,4.781,22.95c-2.869,7.172-4.781,14.821-4.781,22.949c0,23.429,13.866,44.466,34.425,54.028
              c10.041,30.601,38.728,51.638,71.719,51.638H321.3c32.513,0,61.678-21.037,71.719-51.638
              c21.037-9.562,34.425-31.078,34.425-54.028c0-8.128-1.435-15.777-4.781-22.949c2.869-7.172,4.781-15.301,4.781-22.95
              c0-7.172-1.435-14.344-3.825-21.038c3.348-6.693,5.26-14.344,5.26-21.993c3.825-34.425,13.865-52.116,26.775-74.588
              c15.777-27.731,33.469-58.809,33.469-113.793C488.644,88.931,399.712,0,290.222,0z M340.425,515.896
              c-3.347,7.172-10.997,12.432-19.604,12.432h-61.2c-8.606,0-16.256-5.26-19.603-12.432H340.425z M207.028,429.834
              c0-3.347,2.869-6.215,6.216-6.215H367.2c3.347,0,6.215,2.868,6.215,6.215c0,3.348-2.868,6.216-6.215,6.216H213.244
              C209.896,436.05,207.028,433.182,207.028,429.834z M375.328,382.5L375.328,382.5v0.956c0,3.347-2.869,6.216-6.216,6.216H211.331
              c-3.347,0-6.216-2.869-6.216-6.216l-0.956-9.084c-5.737-41.119-20.081-66.46-32.513-88.453
              c-14.344-24.863-26.297-46.378-26.297-87.019c0-79.847,65.025-144.872,144.872-144.872s144.872,64.547,144.872,144.394
              c0,40.641-12.432,62.156-26.297,87.019C395.409,309.347,380.109,336.122,375.328,382.5z M213.244,469.519H367.2
              c3.347,0,6.215,2.869,6.215,6.216s-2.868,6.216-6.215,6.216H213.244c-3.347,0-6.216-2.869-6.216-6.216
              S209.896,469.519,213.244,469.519z";

macro_rules! trans {
    ( $( $fun:expr ),* ) => {
        vec![ $( Box::new($fun) ),* ]
    };
    ( $( $fun:expr ),+ ,) => {
        trans![ $( $fun ),* ]
    };
}

impl Plugin for NodeInGamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ConnectEvent>()
            .add_event::<ChangeInput>()
            .add_event::<DisconnectEvent>()
            .insert_resource(MenuState(MenuStates::Idle))
            .insert_resource(LineResource {
                count: 0.,
                timestep: 0.5,
                update: false,
            })
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .before("interaction2d")
                    .label("level3_node_set")
                    .with_system(ui_node_info_system.system()),
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("level2_node_set")
                    .after("interaction2d")
                    // It's important to run disconnect before systems that delete
                    // nodes (and therefore connectors) because disconnect_event
                    // wants to insert(Free) connectors even if they are queued for
                    // deletion.
                    .with_system(disconnect_event.system().label("disconnect"))
                    .with_system(delete_gate_system.system().after("disconnect"))
                    .with_system(change_input_system.system().after("disconnect"))
                    .with_system(delete_line_system.system().after("disconnect"))
                    .with_system(transition_system.system().label("transition"))
                    .with_system(propagation_system.system().after("transition"))
                    .with_system(highlight_connector_system.system())
                    .with_system(drag_gate_system.system())
                    .with_system(drag_connector_system.system().label("drag_conn_system"))
                    .with_system(connect_nodes.system().after("drag_conn_system"))
                    // Draw Line inserts a new bundle into an entity that might has been
                    // deleted by delete_line_system, i.e. we run it before any deletions
                    // to prevent an segfault.
                    .with_system(
                        draw_line_system
                            .system()
                            .label("draw_line")
                            .before("disconnect"),
                    )
                    .with_system(draw_data_flow.system().after("draw_line"))
                    .with_system(light_bulb_system.system().before("disconnect"))
                    .with_system(toggle_switch_system.system().before("disconnect"))
                    .with_system(line_selection_system.system().after("draw_line")),
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("level1_node_set")
                    .after("level2_node_set")
                    .with_system(open_radial_menu_system.system())
                    .with_system(handle_radial_menu_event_system.system())
                    .with_system(update_radial_menu_system.system()),
            )
            .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(setup.system()));

        info!("NodePlugin loaded");
    }
}

/// The name of an entity.
pub struct Name(String);

/// The input and output states of a logic gate.
///
/// # States
/// `None` - The state is unknown, for example because the gate
/// doesn't get a value for each input.
/// `High` - The sate is high (`1`).
/// `Low` - The state is low (`0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    None,
    High,
    Low,
}

/// Type that maps form an (gate) entity to it's
/// connected inputs.
type TargetMap = HashMap<Entity, Vec<usize>>;

#[derive(Debug, Copy, Clone)]
pub struct NodeRange {
    min: u32,
    max: u32,
}

/// Flag for logic gates.
pub struct Gate {
    pub inputs: u32,
    pub outputs: u32,
    pub in_range: NodeRange,
    pub out_range: NodeRange,
    /// This is a generic gate, i.e. it has not a specific IEEE shape.
    pub generic: bool,
}

const GATE_SIZE: f32 = 128.;
const GATE_WIDTH: f32 = 64.;
const GATE_HEIGHT: f32 = 128.;

struct GateSize {
    width: f32,
    height: f32,
    in_step: f32,
    out_step: f32,
    offset: f32,
}

struct LightBulb {
    state: State,
}

struct ToggleSwitch;
struct Switch;

pub enum SymbolStandard {
    ANSI(PathBuilder),
    BS(Handle<Font>, String, bool), // British System 3939
}

static Z_INDEX: AtomicI32 = AtomicI32::new(1);

impl Gate {
    fn and_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.line_to(Vec2::new(-half, -half));
        path.line_to(Vec2::new(0., -half));
        path.arc(
            Vec2::new(0., 0.),
            Vec2::new(half, half),
            std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }

    fn invert(mut path: PathBuilder) -> PathBuilder {
        let radius = GATE_SIZE * 0.1;
        let half = GATE_SIZE / 2.;

        path.arc(
            Vec2::new(half + radius + 5., 0.),
            Vec2::new(radius, radius),
            std::f32::consts::PI * 2.,
            0.,
        );
        path
    }

    fn nand_gate_path() -> PathBuilder {
        Gate::invert(Gate::and_gate_path())
    }

    fn not_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.line_to(Vec2::new(-half, -half));
        path.line_to(Vec2::new(half, 0.));
        path.close();
        Gate::invert(path)
    }

    fn or_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.arc(
            Vec2::new(-half, 0.),
            Vec2::new(half / 2., half),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(0., -half));
        path.arc(
            Vec2::new(0., 0.),
            Vec2::new(half, half),
            std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }

    fn nor_gate_path() -> PathBuilder {
        Gate::invert(Gate::or_gate_path())
    }

    fn xor_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;
        let mut path = Gate::or_gate_path();

        path.move_to(Vec2::new(-half - 15., half));
        path.arc(
            Vec2::new(-half - 15., 0.),
            Vec2::new(half / 2., half),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(-half - 16., -half));
        path.arc(
            Vec2::new(-half - 16., 0.),
            Vec2::new(half / 2., half),
            std::f32::consts::PI,
            0.,
        );
        path
    }

    fn xnor_gate_path() -> PathBuilder {
        Gate::invert(Gate::xor_gate_path())
    }

    fn toggle_switch_path_a() -> PathBuilder {
        let radius = GATE_SIZE / 4.;
        let mut path = PathBuilder::new();

        path.move_to(Vec2::new(-radius, -radius));
        path.arc(
            Vec2::new(-radius, 0.),
            Vec2::new(radius, radius),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(radius, radius));
        path.arc(
            Vec2::new(radius, 0.),
            Vec2::new(radius, radius),
            -std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }

    fn new_gate_body(position: Vec3, path: PathBuilder) -> ShapeBundle {
        GeometryBuilder::build_as(
            &path.build(),
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default()
                    .with_line_width(6.0)
                    .with_line_join(LineJoin::MiterClip),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        )
    }

    pub fn new_gate(
        commands: &mut Commands,
        name: String,
        x: f32,
        y: f32,
        in_range: NodeRange,
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>,
        standard: SymbolStandard,
    ) {
        let zidx = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let mut sym = None;
        let mut inv = false;

        let (gate, dists, generic) = match standard {
            SymbolStandard::ANSI(path) => (
                Gate::new_gate_body(Vec3::new(x, y, zidx), path),
                Gate::get_distances(
                    in_range.min as f32,
                    out_range.min as f32,
                    GATE_SIZE as f32,
                    GATE_SIZE as f32,
                ),
                false,
            ),
            SymbolStandard::BS(font, symbol, inverted) => {
                sym = Some(
                    commands
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
                            transform: Transform::from_xyz(0., 0., zidx),
                            ..Default::default()
                        })
                        .id(),
                );

                inv = inverted;

                (
                    Gate::new_body(x, y, zidx, GATE_WIDTH, GATE_HEIGHT),
                    Gate::get_distances(
                        in_range.min as f32,
                        out_range.min as f32,
                        GATE_WIDTH as f32,
                        GATE_HEIGHT as f32,
                    ),
                    true,
                )
            }
        };

        let parent = commands
            .spawn_bundle(gate)
            .insert(Gate {
                inputs: in_range.min,
                outputs: out_range.min,
                in_range,
                out_range,
                generic,
            })
            .insert(Name(name))
            .insert(Inputs(vec![State::None; in_range.min as usize]))
            .insert(Outputs(vec![State::None; out_range.min as usize]))
            .insert(Transitions(functions))
            .insert(Targets(vec![HashMap::new(); out_range.min as usize]))
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(dists.width, dists.height),
                NODE_GROUP,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        if let Some(sym) = sym {
            commands.entity(parent).push_children(&[sym]);
        }

        if inv {
            let radius = GATE_SIZE * 0.08;

            let id = commands
                .spawn_bundle(Gate::new_invert(
                    Vec3::new(GATE_WIDTH / 2. + radius, 0., zidx),
                    radius,
                ))
                .id();

            commands.entity(parent).push_children(&[id]);
        }

        let mut entvec: Vec<Entity> = Vec::new();
        for i in 1..=in_range.min {
            entvec.push(Connector::with_line(
                commands,
                Vec3::new(
                    -GATE_SIZE * 0.6,
                    dists.offset + i as f32 * dists.in_step,
                    zidx,
                ),
                GATE_SIZE * 0.1,
                ConnectorType::In,
                (i - 1) as usize,
            ));
        }

        commands.entity(parent).push_children(&entvec);
        entvec.clear();

        for i in 1..=out_range.min {
            entvec.push(Connector::with_line(
                commands,
                Vec3::new(
                    GATE_SIZE * 0.6,
                    dists.offset + i as f32 * dists.out_step,
                    zidx,
                ),
                GATE_SIZE * 0.1,
                ConnectorType::Out,
                (i - 1) as usize,
            ));
        }
        commands.entity(parent).push_children(&entvec);
    }

    fn get_distances(cin: f32, cout: f32, width: f32, _height: f32) -> GateSize {
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

    fn new_body(x: f32, y: f32, z: f32, width: f32, height: f32) -> ShapeBundle {
        let shape = shapes::Rectangle {
            width,
            height,
            ..shapes::Rectangle::default()
        };

        GeometryBuilder::build_as(
            &shape,
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(6.0),
            },
            Transform::from_xyz(x, y, z),
        )
    }

    fn new_invert(position: Vec3, radius: f32) -> ShapeBundle {
        let shape = shapes::Circle {
            radius,
            ..shapes::Circle::default()
        };

        GeometryBuilder::build_as(
            &shape,
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(6.0),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        )
    }

    pub fn constant(
        commands: &mut Commands,
        name: String,
        symbol: String,
        x: f32,
        y: f32,
        state: State,
        font: Handle<Font>,
    ) {
        let dists = Gate::get_distances(1., 1., GATE_WIDTH, GATE_WIDTH);

        let zidx = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let gate = Gate::new_body(x, y, zidx, dists.width, dists.height);

        let sym_text = commands
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
                transform: Transform::from_xyz(0., 0., zidx),
                ..Default::default()
            })
            .id();

        let parent = commands
            .spawn_bundle(gate)
            .insert(Gate {
                inputs: 1,
                outputs: 1,
                in_range: NodeRange { min: 1, max: 1 },
                out_range: NodeRange { min: 1, max: 1 },
                generic: true,
            })
            .insert(Name(name))
            .insert(Inputs(vec![state]))
            .insert(Outputs(vec![State::None]))
            .insert(Transitions(trans![|inputs| inputs[0]]))
            .insert(Targets(vec![HashMap::new()]))
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(dists.width, dists.height),
                NODE_GROUP,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        commands.entity(parent).push_children(&[sym_text]);

        let mut entvec: Vec<Entity> = Vec::new();
        entvec.push(Connector::with_line(
            commands,
            Vec3::new(GATE_SIZE * 0.6, dists.offset + dists.out_step, zidx),
            GATE_SIZE * 0.1,
            ConnectorType::Out,
            0,
        ));
        commands.entity(parent).push_children(&entvec);
    }

    fn light_bulb_bundle(color: Color) -> ShapeBundle {
        GeometryBuilder::build_as(
            &SvgPathShape {
                svg_doc_size_in_px: Vec2::new(580.922, 580.922),
                svg_path_string: LIGHT_BULB_PATH.to_string(),
            },
            ShapeColors::outlined(color, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(8.0),
            },
            Transform::from_scale(Vec3::new(0.22, 0.22, 0.22)),
        )
    }

    pub fn light_bulb(commands: &mut Commands, x: f32, y: f32) {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let parent = commands
            .spawn()
            .insert(Transform::from_xyz(x, y, z))
            .insert(GlobalTransform::from_xyz(x, y, z))
            .insert(LightBulb { state: State::None })
            .insert(Name("Light Bulb".to_string()))
            .insert(Inputs(vec![State::None]))
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE, GATE_SIZE),
                NODE_GROUP,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        let bulb = commands
            .spawn_bundle(Gate::light_bulb_bundle(Color::WHITE))
            .id();

        let child = Connector::with_line(
            commands,
            Vec3::new(0., -GATE_SIZE * 0.7, 0.),
            GATE_SIZE * 0.1,
            ConnectorType::In,
            0,
        );

        commands.entity(parent).push_children(&vec![bulb, child]);
    }

    pub fn toggle_switch(commands: &mut Commands, x: f32, y: f32) {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let switch = GeometryBuilder::build_as(
            &Gate::toggle_switch_path_a().build(),
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(8.0),
            },
            Transform::from_xyz(x, y, z),
        );

        let parent = commands
            .spawn_bundle(switch)
            .insert(ToggleSwitch)
            .insert(Name("Toggle Switch".to_string()))
            .insert(Inputs(vec![State::Low]))
            .insert(Outputs(vec![State::Low]))
            .insert(Transitions(trans![|inputs| inputs[0]]))
            .insert(Targets(vec![HashMap::new()]))
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE, GATE_SIZE),
                NODE_GROUP,
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

        let nod = GeometryBuilder::build_as(
            &shapes::Circle {
                radius: GATE_SIZE / 4.,
                center: Vec2::new(0., 0.),
            },
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(8.0),
            },
            Transform::from_xyz(-GATE_SIZE / 4., 0., 1.),
        );

        let nod_child = commands
            .spawn_bundle(nod)
            .insert(Switch)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE / 2., GATE_SIZE / 2.),
                NODE_GROUP,
            ))
            .id();

        commands
            .entity(parent)
            .push_children(&vec![child, nod_child]);
    }

    pub fn not_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "NOT Gate".to_string(),
            position.x,
            position.y,
            NodeRange { min: 1, max: 1 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
            },],
            //SymbolStandard::ANSI(Gate::not_gate_path()),
            SymbolStandard::BS(font, "1".to_string(), true),
        );
    }

    pub fn and_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "AND Gate".to_string(),
            position.x,
            position.y,
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
            //SymbolStandard::ANSI(Gate::and_gate_path()),
            SymbolStandard::BS(font, "&".to_string(), false),
        );
    }

    pub fn nand_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "NAND Gate".to_string(),
            position.x,
            position.y,
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
            //SymbolStandard::ANSI(Gate::nand_gate_path()),
            SymbolStandard::BS(font, "&".to_string(), true),
        );
    }

    pub fn or_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "OR Gate".to_string(),
            //"≥1".to_string(),
            position.x,
            position.y,
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
            //SymbolStandard::ANSI(Gate::or_gate_path()),
            SymbolStandard::BS(font, "≥1".to_string(), false),
        );
    }

    pub fn nor_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "NOR Gate".to_string(),
            position.x,
            position.y,
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
            //SymbolStandard::ANSI(Gate::nor_gate_path()),
            SymbolStandard::BS(font, "≥1".to_string(), true),
        );
    }

    pub fn xor_gate(commands: &mut Commands, position: Vec2, font: Handle<Font>) {
        Gate::new_gate(
            commands,
            "XOR Gate".to_string(),
            position.x,
            position.y,
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
            //SymbolStandard::ANSI(Gate::xor_gate_path()),
            SymbolStandard::BS(font, "=1".to_string(), false),
        );
    }

    pub fn high_const(commands: &mut Commands, font: Handle<Font>, position: Vec2) {
        Gate::constant(
            commands,
            "HIGH Const".to_string(),
            "1".to_string(),
            position.x,
            position.y,
            State::High,
            font,
        );
    }

    pub fn low_const(commands: &mut Commands, font: Handle<Font>, position: Vec2) {
        Gate::constant(
            commands,
            "LOW Const".to_string(),
            "0".to_string(),
            position.x,
            position.y,
            State::Low,
            font,
        );
    }
}

/// Input values of a logical node, e.g. a gate.
pub struct Inputs(Vec<State>);

/// Output values of a logical node, e.g. a gate.
pub struct Outputs(Vec<State>);

/// A set of transition functions `f: Inputs -> State`.
///
/// For a logic node, e.g. a gate, there should be a transition function
/// for each output.
pub struct Transitions(Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>);

/// A vector that maps from outputs to connected nodes.
///
/// For a logic node, e.g. a gate, there should be a vector entry for
/// each output.
pub struct Targets(Vec<TargetMap>);

/// System for calculating the state of each output using the corresponding
/// transition functions.
fn transition_system(mut query: Query<(&Inputs, &Transitions, &mut Outputs)>) {
    for (inputs, transitions, mut outputs) in query.iter_mut() {
        for i in 0..transitions.0.len() {
            outputs.0[i] = transitions.0[i](&inputs.0);
        }
    }
}

/// System for writing the calculated output states to the inputs of each connected node.
fn propagation_system(from_query: Query<(&Outputs, &Targets)>, mut to_query: Query<&mut Inputs>) {
    for (outputs, targets) in from_query.iter() {
        for i in 0..outputs.0.len() {
            for (&entity, idxvec) in &targets.0[i] {
                if let Ok(mut inputs) = to_query.get_mut(entity) {
                    for &j in idxvec {
                        if j < inputs.0.len() {
                            inputs.0[j] = outputs.0[i];
                        }
                    }
                } else {
                    error!("Could not query inputs of given entity ");
                }
            }
        }
    }
}

fn setup(mut _commands: Commands, _font: Res<FontAssets>, _gate: Res<GateAssets>) {}

fn drag_gate_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    q_dragged: Query<
        Entity,
        (
            With<Drag>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>)>,
        ),
    >,
) {
    if mb.just_released(MouseButton::Left) {
        for dragged_gate in q_dragged.iter() {
            commands.entity(dragged_gate).remove::<Drag>();
        }
    }
}

fn delete_gate_system(
    mut commands: Commands,
    input_keyboard: Res<Input<KeyCode>>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    q_gate: Query<
        (Entity, &Children),
        (
            With<Selected>,
            Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>)>,
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
                    for &connection in &conns.0 {
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

fn light_bulb_system(
    mut commands: Commands,
    mut q_light: Query<(&Children, &Inputs, &mut LightBulb)>,
    q_bulb: Query<Entity, Without<Connector>>,
) {
    for (children, inputs, mut light) in q_light.iter_mut() {
        if inputs.0[0] != light.state {
            let color = match inputs.0[0] {
                State::High => Color::BLUE,
                _ => Color::WHITE,
            };

            for &child in children.iter() {
                if let Ok(entity) = q_bulb.get(child) {
                    commands.entity(entity).remove_bundle::<ShapeBundle>();
                    commands
                        .entity(entity)
                        .insert_bundle(Gate::light_bulb_bundle(color));
                }
            }

            light.state = inputs.0[0];
        }
    }
}

fn toggle_switch_system(
    mut commands: Commands,
    mut q_outputs: Query<&mut Inputs>,
    mut q_switch: Query<(&Parent, &mut Transform), (With<Hover>, With<Switch>)>,
    mb: Res<Input<MouseButton>>,
) {
    if mb.just_pressed(MouseButton::Left) {
        for (parent, mut transform) in q_switch.iter_mut() {
            if let Ok(mut inputs) = q_outputs.get_mut(parent.0) {
                let next = match inputs.0[0] {
                    State::High => {
                        transform.translation.x -= GATE_SIZE / 2.;
                        State::Low
                    }
                    _ => {
                        transform.translation.x += GATE_SIZE / 2.;
                        State::High
                    }
                };
                inputs.0[0] = next;
            }
        }
    }
}

// ############################# Connector ##############################################

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectorType {
    In,
    Out,
}

/// A connector acts as the interface between two nodes, e.g. a logic gate.
pub struct Connector {
    /// The type of the connector.
    ctype: ConnectorType,
    /// Its index in context of a logical node.
    index: usize,
}

/// Connection lines connected to this connector.
pub struct Connections(Vec<Entity>);

pub struct Free;

impl Connector {
    /// Create a new connector for a logic node.
    pub fn new(
        commands: &mut Commands,
        position: Vec3,
        radius: f32,
        ctype: ConnectorType,
        index: usize,
    ) -> Entity {
        let circle = shapes::Circle {
            radius: radius,
            center: Vec2::new(0., 0.),
        };

        let connector = GeometryBuilder::build_as(
            &circle,
            ShapeColors::outlined(Color::WHITE, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(5.0),
            },
            Transform::from_xyz(position.x, position.y, 0.),
        );

        commands
            .spawn_bundle(connector)
            .insert(Connector { ctype, index })
            .insert(Connections(Vec::new()))
            .insert(Free)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(radius * 2., radius * 2.),
                CONNECTOR_GROUP,
            ))
            .insert(Selectable)
            .insert(Draggable { update: false })
            .id()
    }

    pub fn with_line(
        commands: &mut Commands,
        position: Vec3,
        radius: f32,
        ctype: ConnectorType,
        index: usize,
    ) -> Entity {
        let id = Connector::new(commands, position, radius, ctype, index);
        let line = shapes::Line(Vec2::new(-position.x, 0.), Vec2::new(0., 0.));
        let line_conn = GeometryBuilder::build_as(
            &line,
            ShapeColors::new(Color::BLACK),
            DrawMode::Stroke(StrokeOptions::default().with_line_width(6.0)),
            Transform::from_xyz(0., 0., -1.),
        );

        let line_id = commands.spawn_bundle(line_conn).id();
        commands.entity(id).push_children(&[line_id]);
        id
    }
}

/// Highlight a connector by increasing its radius when the mouse
/// hovers over it.
fn highlight_connector_system(
    // We need all connectors the mouse hovers over.
    mut q_hover: Query<&mut Transform, (With<Hover>, With<Connector>)>,
    mut q2_hover: Query<&mut Transform, (Without<Hover>, With<Connector>)>,
) {
    for mut transform in q_hover.iter_mut() {
        transform.scale = Vec3::new(1.2, 1.2, transform.scale.z);
    }

    for mut transform in q2_hover.iter_mut() {
        transform.scale = Vec3::new(1.0, 1.0, transform.scale.z);
    }
}

/// A line shown when the user clicks and drags from a connector.
/// It's expected that there is atmost one ConnectionLineIndicator
/// present.
pub struct ConnectionLineIndicator;

fn drag_connector_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    mw: Res<MouseWorldPos>,
    // ID and transform of the connector we drag from.
    q_dragged: Query<(Entity, &GlobalTransform, &Connector), (With<Drag>, With<Free>)>,
    // The visual connection line indicator to update.
    q_conn_line: Query<Entity, With<ConnectionLineIndicator>>,
    // Posible free connector the mouse currently hovers over.
    q_drop: Query<(Entity, &Connector), (With<Hover>, With<Free>)>,
    mut ev_connect: EventWriter<ConnectEvent>,
) {
    if let Ok((entity, transform, connector)) = q_dragged.single() {
        // If the LMB is released we check if we can connect two connectors.
        if mb.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Drag>();

            // We dont need the visual connection line any more.
            // There will be another system responsible for
            // drawing the connections between nodes.
            if let Ok(conn_line) = q_conn_line.single() {
                commands.entity(conn_line).despawn_recursive();
            }

            // Try to connect input and output.
            if let Ok((drop_target, drop_connector)) = q_drop.single() {
                eprintln!("drop");
                // One can only connect an input to an output.
                if connector.ctype != drop_connector.ctype {
                    // Send connection event.
                    match connector.ctype {
                        ConnectorType::In => {
                            ev_connect.send(ConnectEvent {
                                output: drop_target,
                                output_index: drop_connector.index,
                                input: entity,
                                input_index: connector.index,
                            });
                        }
                        ConnectorType::Out => {
                            ev_connect.send(ConnectEvent {
                                output: entity,
                                output_index: connector.index,
                                input: drop_target,
                                input_index: drop_connector.index,
                            });
                        }
                    }
                }
            }
        } else {
            // While LMB is being pressed, draw the line from the node clicked on
            // to the mouse cursor.
            let conn_entity = if let Ok(conn_line) = q_conn_line.single() {
                commands.entity(conn_line).remove_bundle::<ShapeBundle>();
                conn_line
            } else {
                commands.spawn().insert(ConnectionLineIndicator).id()
            };

            let shape = shapes::Line(
                Vec2::new(transform.translation.x, transform.translation.y),
                Vec2::new(mw.x, mw.y),
            );

            let line = GeometryBuilder::build_as(
                &shape,
                ShapeColors::outlined(Color::TEAL, Color::BLACK),
                DrawMode::Outlined {
                    fill_options: FillOptions::default(),
                    outline_options: StrokeOptions::default().with_line_width(10.0),
                },
                Transform::from_xyz(0., 0., 1.),
            );

            commands.entity(conn_entity).insert_bundle(line);
        }
    }
}

struct ConnectEvent {
    output: Entity,
    output_index: usize,
    input: Entity,
    input_index: usize,
}

/// Handle incomming connection events.
fn connect_nodes(
    mut commands: Commands,
    mut ev_connect: EventReader<ConnectEvent>,
    mut q_conns: Query<(&Parent, &mut Connections), ()>,
    mut q_parent: Query<&mut Targets>,
    q_transform: Query<&GlobalTransform, ()>,
) {
    for ev in ev_connect.iter() {
        let line = ConnectionLine::new(
            &mut commands,
            ConnInfo {
                entity: ev.output,
                index: ev.output_index,
            },
            ConnInfo {
                entity: ev.input,
                index: ev.input_index,
            },
            (
                q_transform.get(ev.output).unwrap().translation,
                q_transform.get(ev.input).unwrap().translation,
            ),
        );

        let input_parent = if let Ok((parent, mut connections)) = q_conns.get_mut(ev.input) {
            connections.0.push(line);
            parent.0
        } else {
            continue;
        };
        commands.entity(ev.input).remove::<Free>();

        if let Ok((parent, mut connections)) = q_conns.get_mut(ev.output) {
            connections.0.push(line);

            if let Ok(mut targets) = q_parent.get_mut(parent.0) {
                targets.0[ev.output_index]
                    .entry(input_parent)
                    .or_insert(Vec::new())
                    .push(ev.input_index);
            }
        }
    }
}

struct DisconnectEvent {
    connection: Entity,
    in_parent: Option<Entity>,
}

fn disconnect_event(
    mut commands: Commands,
    mut ev_disconnect: EventReader<DisconnectEvent>,
    q_line: Query<&ConnectionLine>,
    mut q_conn: Query<(&Parent, Entity, &mut Connections)>,
    mut q_parent: Query<&mut Targets>,
    mut q_input: Query<&mut Inputs>,
) {
    for ev in ev_disconnect.iter() {
        if let Ok(line) = q_line.get(ev.connection) {
            let in_parent: Option<Entity>;

            // Unlink input connector (right hand side)
            if let Ok((parent_in, entity_in, mut connections_in)) =
                q_conn.get_mut(line.input.entity)
            {
                in_parent = Some(parent_in.0);

                // Reset input state of the given connector.
                if let Ok(mut inputs) = q_input.get_mut(parent_in.0) {
                    inputs.0[line.input.index] = State::None;
                }

                // Clear the input line from the vector and
                // mark the connector as free.
                connections_in.0.clear();
                commands.entity(entity_in).insert(Free);
            } else {
                in_parent = ev.in_parent;
            }

            // Unlink output connector (left hand side)
            if let Ok((parent_out, _entity_out, mut connections_out)) =
                q_conn.get_mut(line.output.entity)
            {
                let parent = in_parent.expect("There should always bee a parent set");

                // Find and remove the given connection line.
                if let Some(idx) = connections_out.0.iter().position(|x| *x == ev.connection) {
                    connections_out.0.remove(idx);
                }

                // Unlink propagation target.
                // Find the index of the input connector within the
                // target map of the gate the output connector belongs
                // to and remove the associated entry.
                if let Ok(mut targets) = q_parent.get_mut(parent_out.0) {
                    let size = targets.0[line.output.index]
                        .get(&parent)
                        .expect("Should have associated entry")
                        .len();

                    if size > 1 {
                        if let Some(index) = targets.0[line.output.index]
                            .get_mut(&parent)
                            .expect("Should have associated entry")
                            .iter()
                            .position(|x| *x == line.input.index)
                        {
                            targets.0[line.output.index]
                                .get_mut(&parent)
                                .expect("Should have associated entry")
                                .remove(index);
                        }
                    } else {
                        targets.0[line.output.index].remove(&parent);
                    }
                }
            }

            // Finally remove the connection line itself.
            commands.entity(ev.connection).despawn_recursive();
        }
    }
}

// ############################# Connection Line ########################################

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnInfo {
    entity: Entity,
    index: usize,
}

pub struct ConnectionLine {
    output: ConnInfo,
    via: Vec<Vec2>,
    input: ConnInfo,
}

impl ConnectionLine {
    pub fn new(
        commands: &mut Commands,
        output: ConnInfo,
        input: ConnInfo,
        positions: (Vec3, Vec3),
    ) -> Entity {
        commands
            .spawn()
            .insert(ConnectionLine {
                output,
                via: ConnectionLine::calculate_nodes(
                    positions.0.x,
                    positions.0.y,
                    positions.1.x,
                    positions.1.y,
                ),
                input,
            })
            .id()
    }

    /// Calculate the nodes of a path between two points.
    pub fn calculate_nodes(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<Vec2> {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dx2 = dx / 2.;
        let dy2 = dy / 2.;
        let point1 = Vec2::new(x1, y1);
        let point2 = if dx >= 0. {
            Vec2::new(x1 + dx2, y1)
        } else {
            Vec2::new(x1, y1 + dy2)
        };
        let point3 = if dx >= 0. {
            Vec2::new(x1 + dx2, y1 + dy)
        } else {
            Vec2::new(x1 + dx, y1 + dy2)
        };
        let point4 = Vec2::new(x1 + dx, y1 + dy);

        vec![point1, point2, point3, point4]
    }
}

fn draw_line_system(
    mut commands: Commands,
    mut q_line: Query<(Entity, &mut ConnectionLine), ()>,
    q_transform: Query<(&Parent, &Connector, &GlobalTransform), ()>,
    q_outputs: Query<&Outputs, ()>,
    mut lr: ResMut<LineResource>,
    time: Res<Time>,
) {
    lr.count += time.delta_seconds();

    for (entity, mut conn_line) in q_line.iter_mut() {
        if let Ok((t_parent, t_conn, t_from)) = q_transform.get(conn_line.output.entity) {
            // Set connection line color based on the value of the output.
            let color = if let Ok(outputs) = q_outputs.get(t_parent.0) {
                match outputs.0[t_conn.index] {
                    State::None => Color::RED,
                    State::High => Color::BLUE,
                    State::Low => Color::BLACK,
                }
            } else {
                Color::BLACK
            };

            if let Ok((_, _, t_to)) = q_transform.get(conn_line.input.entity) {
                let via = ConnectionLine::calculate_nodes(
                    t_from.translation.x,
                    t_from.translation.y,
                    t_to.translation.x,
                    t_to.translation.y,
                );
                let l = ((via[3].x - via[0].x).powi(2) + (via[3].y - via[0].y).powi(2)).sqrt();

                // Remove current line path.
                commands.entity(entity).remove_bundle::<ShapeBundle>();

                // Create new path.
                let mut path = PathBuilder::new();
                path.move_to(via[0]);
                path.cubic_bezier_to(
                    via[1],
                    via[2],
                    via[3],
                );

                commands
                    .entity(entity)
                    .insert_bundle(GeometryBuilder::build_as(
                        &path.build(),
                        ShapeColors::new(color),
                        DrawMode::Stroke(
                            StrokeOptions::default()
                                .with_line_width(8.0)
                                .with_line_join(LineJoin::Bevel),
                        ),
                        Transform::from_xyz(0., 0., 0.),
                    ));

                conn_line.via = via;

                if color == Color::BLUE && lr.count >= lr.timestep {
                    let id = commands
                        .spawn_bundle(
                            GeometryBuilder::build_as(
                                &shapes::Circle {
                                    radius: 3.,
                                    center: Vec2::new(0., 0.),
                                },
                                ShapeColors::outlined(Color::WHITE, Color::WHITE),
                                DrawMode::Outlined {
                                    fill_options: FillOptions::default(),
                                    outline_options: StrokeOptions::default()
                                        .with_line_width(1.0)
                                        .with_line_join(LineJoin::MiterClip),
                                },
                                Transform::from_xyz(t_from.translation.x, t_from.translation.y, 1.),
                            )
                        )
                        .insert(DataPoint {
                            stepsize: 1. / (l / 250.),
                            steps: 0.,
                        }).id();
                    
                    commands.entity(entity).push_children(&[id]);
                }
            }
        }
    }

    if lr.count >= lr.timestep {
        lr.count = 0.;
    }
}

fn line_selection_system(
    mut commands: Commands,
    mw: Res<MouseWorldPos>,
    mb: Res<Input<MouseButton>>,
    q_line: Query<(Entity, &ConnectionLine)>,
    q_selected: Query<Entity, With<Selected>>,
) {
    if mb.just_pressed(MouseButton::Left) {
        for (entity, line) in q_line.iter() {
            if let Some(_) = t_for_point(
                Vec2::new(mw.x, mw.y), 
                line.via[0].clone(), 
                line.via[1].clone(),
                line.via[2].clone(),
                line.via[3].clone()
            ) {
                eprintln!("in");
                commands.entity(entity).insert(Selected);
                break;
            }
        }
    }
}

fn delete_line_system(
    input_keyboard: Res<Input<KeyCode>>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    q_line: Query<Entity, (With<Selected>, With<ConnectionLine>)>,
) {
    if input_keyboard.just_pressed(KeyCode::Delete) {
        for entity in q_line.iter() {
            eprintln!("delete {:?}", entity);
            ev_disconnect.send(DisconnectEvent {
                connection: entity,
                in_parent: None,
            });
        }
    }
}

/// Sameple the cubic bezier curve, defined by s` (start),
/// `c1` (control point 1), `c2` (control point 2) and `e` (end),
/// at `t` (expecting t between 0 and 1);
fn qubic_bezier_point(t: f32, s: Vec2, c1: Vec2, c2: Vec2, e: Vec2) -> Vec2 {
    let u = 1. - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut p = s * uuu;
    p += c1 * 3. * uu * t;
    p += c2 * 3. * u * tt;
    p += e * ttt;
    p
}

/// Solve t for a point `xy` on a qubic bezier curve defined by `s` (start),
/// `c1` (control point 1), `c2` (control point 2) and `e` (end).
///
/// This is just a approximation and can be used to check if a user clicked
/// on a qubic bezier curve.
fn t_for_point(xy: Vec2, s: Vec2, c1: Vec2, c2: Vec2, e: Vec2) -> Option<f32> {
    use lyon_geom::*;
    
    const epsilon: f32 = 16.;
    let c = CubicBezierSegment {
        from: Point::new(s.x, s.y),
        ctrl1: Point::new(c1.x, c1.y),
        ctrl2: Point::new(c2.x, c2.y),
        to: Point::new(e.x, e.y),
    };

    let possible_t_values_x = c.solve_t_for_x(xy.x);
    let possible_t_values_y = c.solve_t_for_y(xy.y);

    for t in possible_t_values_x {
        if t >= -0.001 && t <= 1.001 {
            let p = c.sample(t);

            let offset = p - Point::new(xy.x, xy.y);
            let dot = offset.x * offset.x + offset.y * offset.y;
            if dot <= epsilon * epsilon {
                return Some(t);
            }
        }
    }

    None
}

struct LineResource {
    count: f32,
    timestep: f32,
    update: bool,
}

struct DataPoint {
    stepsize: f32,
    steps: f32,
}

fn draw_data_flow(
    mut commands: Commands,
    time: Res<Time>,
    mut q_point: Query<(Entity, &Parent, &mut Transform, &mut DataPoint)>,
    q_line: Query<&ConnectionLine>, 
) {
    for (entity, parent, mut transform, mut data) in q_point.iter_mut() {
        if let Ok(line) = q_line.get(parent.0) {
            let l = ((line.via[3].x - line.via[0].x).powi(2) + (line.via[3].y - line.via[0].y).powi(2)).sqrt();
            data.steps += (1. / (l / 300.)) * time.delta_seconds();

            if data.steps >= 1.0 {
                commands.entity(entity).despawn_recursive();
            } else {
                let p = qubic_bezier_point(
                    data.steps, 
                    line.via[0].clone(), 
                    line.via[1].clone(),
                    line.via[2].clone(),
                    line.via[3].clone()
                );

                transform.translation.x = p.x; 
                transform.translation.y = p.y; 
            }
        }
    }
}


// ############################# User Interface #########################################
#[derive(AssetCollection)]
pub struct GateAssets {
    #[asset(color_material)]
    #[asset(path = "gates/not.png")]
    pub not: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/and.png")]
    pub and: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/nand.png")]
    pub nand: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/or.png")]
    pub or: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/nor.png")]
    pub nor: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/xor.png")]
    pub xor: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/back.png")]
    pub back: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/close.png")]
    pub close: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/circuit.png")]
    pub circuit: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/in.png")]
    pub inputs: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/out.png")]
    pub outputs: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/high.png")]
    pub high: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/low.png")]
    pub low: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/toggle.png")]
    pub toggle: Handle<ColorMaterial>,

    #[asset(color_material)]
    #[asset(path = "gates/bulb.png")]
    pub bulb: Handle<ColorMaterial>,
}

#[derive(Debug, Eq, PartialEq)]
enum MenuStates {
    Idle,
    Select,
    LogicGates,
    Inputs,
    Outputs,
}

struct MenuState(MenuStates);

fn open_radial_menu_system(
    mb: Res<Input<MouseButton>>,
    mw: Res<MouseWorldPos>,
    assets: Res<GateAssets>,
    mut ms: ResMut<MenuState>,
    mut ev_open: EventWriter<OpenMenuEvent>,
) {
    if mb.just_pressed(MouseButton::Right) && ms.0 == MenuStates::Idle {
        ev_open.send(OpenMenuEvent {
            position: Vec2::new(mw.x, mw.y),
            mouse_button: MouseButton::Left,
            items: vec![
                (
                    assets.close.clone(),
                    "close".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.circuit.clone(),
                    "Show Logic\nGates".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.inputs.clone(),
                    "Show Input\nControls".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.outputs.clone(),
                    "Show Output\nControls".to_string(),
                    Vec2::new(80., 80.),
                ),
            ],
        });

        ms.0 = MenuStates::Select;
    }
}

fn update_radial_menu_system(
    mw: Res<MouseWorldPos>,
    mut ev_update: EventWriter<UpdateCursorPositionEvent>,
) {
    ev_update.send(UpdateCursorPositionEvent(mw.0));
}

fn handle_radial_menu_event_system(
    mut commands: Commands,
    mut ev_radial: EventReader<PropagateSelectionEvent>,
    mut ev_open: EventWriter<OpenMenuEvent>,
    font: Res<FontAssets>,
    assets: Res<GateAssets>,
    mut ms: ResMut<MenuState>,
) {
    for ev in ev_radial.iter() {
        match ms.0 {
            MenuStates::Select => match ev.id {
                1 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.and.clone(),
                                "AND gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                            (
                                assets.nand.clone(),
                                "NAND gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                            (
                                assets.or.clone(),
                                "OR gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                            (
                                assets.nor.clone(),
                                "NOR gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                            (
                                assets.not.clone(),
                                "NOT gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                            (
                                assets.xor.clone(),
                                "XOR gate".to_string(),
                                Vec2::new(100., 40.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::LogicGates;
                }
                2 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.high.clone(),
                                "HIGH const".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.low.clone(),
                                "LOW const".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.toggle.clone(),
                                "Toggle Switch".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::Inputs;
                }
                3 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.bulb.clone(),
                                "Light Bulb".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::Outputs;
                }
                _ => {
                    ms.0 = MenuStates::Idle;
                }
            },
            MenuStates::LogicGates => match ev.id {
                1 => {
                    Gate::and_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    Gate::nand_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    Gate::or_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                4 => {
                    Gate::nor_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                5 => {
                    Gate::not_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                6 => {
                    Gate::xor_gate(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Inputs => match ev.id {
                1 => {
                    Gate::high_const(&mut commands, font.main.clone(), ev.position);
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    Gate::low_const(&mut commands, font.main.clone(), ev.position);
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    Gate::toggle_switch(&mut commands, ev.position.x, ev.position.y);
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Outputs => match ev.id {
                1 => {
                    Gate::light_bulb(&mut commands, ev.position.x, ev.position.y);
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Idle => {} // This should never happen
        }
    }
}

struct ChangeInput {
    gate: Entity,
    to: u32,
}

fn ui_node_info_system(
    egui_context: ResMut<EguiContext>,
    q_gate: Query<(Entity, &Name, Option<&Gate>), With<Selected>>,
    mut ev_change: EventWriter<ChangeInput>,
    mut mb: ResMut<Input<MouseButton>>,
) {
    if let Ok((entity, name, gate)) = q_gate.single() {
        if let Some(response) = egui::Window::new(&name.0)
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-5., -5.))
            .show(egui_context.ctx(), |ui| {
                ui.label(&name.0);

                if let Some(gate) = gate {
                    if gate.in_range.min != gate.in_range.max {
                        if ui
                            .horizontal(|ui| {
                                ui.label("Input Count: ");
                                if ui.button("➖").clicked() {
                                    if gate.inputs > gate.in_range.min {
                                        ev_change.send(ChangeInput {
                                            gate: entity,
                                            to: gate.inputs - 1,
                                        });
                                    }
                                }
                                ui.label(format!("{}", gate.inputs));
                                if ui.button("➕").clicked() {
                                    if gate.inputs < gate.in_range.max {
                                        ev_change.send(ChangeInput {
                                            gate: entity,
                                            to: gate.inputs + 1,
                                        });
                                    }
                                }
                            })
                            .response
                            .hovered()
                        {
                            mb.reset(MouseButton::Left);
                        }
                    }
                }
            })
        {
            if response.response.hovered() {
                mb.reset(MouseButton::Left);
            }
        }
    }

    egui::TopBottomPanel::top("side").show(egui_context.ctx(), |ui| {
        ui.label("Hello World");
    });
}

fn change_input_system(
    mut commands: Commands,
    mut ev_connect: EventReader<ChangeInput>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    mut q_gate: Query<(
        Entity,
        &mut Gate,
        &mut Inputs,
        &mut Interactable,
        &GlobalTransform,
    )>,
    q_connectors: Query<&Children>,
    mut q_connector: Query<(&mut Connector, &mut Transform, &Connections)>,
) {
    for ev in ev_connect.iter() {
        if let Ok((gent, mut gate, mut inputs, mut interact, transform)) = q_gate.get_mut(ev.gate) {
            // Update input count
            gate.inputs = ev.to;

            let translation = transform.translation;

            // Update input vector
            inputs.0.resize(gate.inputs as usize, State::None);

            // If the logic component is generic it has a box as body.
            // We are going to resize it in relation to the number
            // of input connectors.
            let dists = if gate.generic {
                let dists = Gate::get_distances(
                    gate.inputs as f32,
                    gate.outputs as f32,
                    GATE_WIDTH,
                    GATE_HEIGHT,
                );

                // Update bounding box
                interact.update_size(0., 0., dists.width, dists.height);

                let gate = Gate::new_body(
                    translation.x,
                    translation.y,
                    translation.z,
                    dists.width,
                    dists.height,
                );

                // Update body
                commands.entity(ev.gate).remove_bundle::<ShapeBundle>();
                commands.entity(ev.gate).insert_bundle(gate);

                dists
            } else {
                Gate::get_distances(
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
                                for &c in &conns.0 {
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
                    (i - 1),
                ));
            }
            if !entvec.is_empty() {
                commands.entity(ev.gate).push_children(&entvec);
            }
        }
    }
}
