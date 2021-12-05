use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;
use nodus::world2d::interaction2d::*;
use std::sync::atomic::{AtomicI32, Ordering};
 use nodus::world2d::camera2d::MouseWorldPos;

pub struct NodePlugin;

const NODE_GROUP: u32 = 1;
const CONNECTOR_GROUP: u32 = 2;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut AppBuilder) {
        // add things to the app here
        //.add_system(hello_world.system())
        //.add_system(greet_node.system())
        app.add_startup_system(setup.system())
            .add_stage_after(
                CoreStage::Update,
                NodeStages::Update,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                NodeStages::Update,
                transition_system.system().label(NodeLabels::Transition)
            )
            .add_system_to_stage(
                NodeStages::Update,
                propagation_system.system().after(NodeLabels::Transition)
            )
            .add_system(highlight_connector_system.system())
            .add_system(drag_gate.system())
            .add_system(drag_connector.system());
        
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

/// System stages to group systems related to the
/// node module.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum NodeStages {
    Update
}

/// Labels for the different systems of this module.
/// The labels are used to force an explicit ordering
/// between the systems when neccessary.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum NodeLabels {
    Transition
}

/// Type that maps form an (gate) entity to it's 
/// connected inputs.
type TargetMap = HashMap<Entity, Vec<usize>>;

#[derive(Debug, Copy, Clone)]
pub struct NodeRange {
    min: u32,
    max: u32
}

/// Flag for logic gates.
pub struct Gate {
    pub inputs: u32,
    pub outputs: u32,
    pub in_range: NodeRange,
    pub out_range: NodeRange,
}

impl Gate {

    pub fn new(
        commands: &mut Commands, 
        name: String,
        x: f32, y: f32, 
        in_range: NodeRange, 
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>
    ) {
        static Z_INDEX: AtomicI32 = AtomicI32::new(1);
        const GATE_SIZE: f32 = 100.;
        
        let factor = if in_range.min >= out_range.min { in_range.min } else { out_range.min };
        let width = GATE_SIZE;
        let height = GATE_SIZE + if factor > 2 {
            (factor - 1) as f32 * GATE_SIZE / 2.
        } else { 0. };
        let in_step = -(height / (in_range.min as f32 + 1.));
        let out_step = -(height / (out_range.min as f32 + 1.));
        let offset = height / 2.;

        let zidx = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let shape = shapes::Rectangle {
            width,
            height,
            ..shapes::Rectangle::default()
        };
        let gate = GeometryBuilder::build_as(
            &shape,
            ShapeColors::outlined(Color::TEAL, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(10.0),
            },
            Transform::from_xyz(x, y, zidx),
        );
        let parent = commands
            .spawn_bundle(gate)
            .insert(Gate { 
                inputs: in_range.min,
                outputs: out_range.min,
                in_range,
                out_range
            })
            .insert(Name(name))
            .insert(Inputs(vec![State::None; in_range.min as usize]))
            .insert(Outputs(vec![State::None; out_range.min as usize]))
            .insert(Transitions(functions))
            .insert(Targets(vec![HashMap::new(); out_range.min as usize]))
            .insert(Interactable::new(Vec2::new(0., 0.), Vec2::new(width, height), NODE_GROUP))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();
        
        let mut entvec: Vec<Entity> = Vec::new();
        for i in 1..=in_range.min {
            entvec.push(Connector::new(commands, 
                                       Vec3::new(-75., offset + i as f32 * in_step, zidx), 
                                       12., 
                                       ConnectorType::In));
        }

        commands.entity(parent).push_children(&entvec);
        entvec.clear();

        for i in 1..=out_range.min {
            entvec.push(Connector::new(commands, 
                                       Vec3::new(75., offset + i as f32 * out_step, zidx), 
                                       12., 
                                       ConnectorType::Out));
        }
        commands.entity(parent).push_children(&entvec);
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
            for (entity, idxvec) in &targets.0[i] {
                if let Ok(mut inputs) = to_query.get_component_mut::<Inputs>(*entity) {
                    for j in idxvec {
                        inputs.0[*j] = outputs.0[i];
                    }
                } else {
                    error!("Could not query inputs of given entity ");
                }
            }
        }
    }
}

fn setup(mut commands: Commands) {
    Gate::new(&mut commands, 
              "NOT Gate".to_string(), 
              0., 0., 
              NodeRange { min: 1, max: 1 },
              NodeRange { min: 1, max: 1 },
              vec![Box::new(|inputs| {
                    match inputs[0] {
                        State::None => State::None,
                        State::Low => State::High,
                        State::High => State::Low,
                    }
              })]);

    Gate::new(&mut commands, 
              "AND Gate".to_string(), 
              250., 0., 
              NodeRange { min: 2, max: 16 },
              NodeRange { min: 1, max: 1 },
              vec![Box::new(|inputs| {
                  let mut ret = State::High;
                  for i in inputs {
                    match i {
                        State::None => { ret = State::None; },
                        State::Low => { ret = State::Low; break; },
                        State::High => { },
                    }
                  }
                  ret
              })]);
}

fn add_gate(commands: &mut Commands, x: f32, y: f32, width: f32, height: f32) {
}

fn drag_gate(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    q_dragged: Query<Entity, (With<Drag>, With<Gate>)>
) {
    if mb.just_released(MouseButton::Left) {
        for dragged_gate in q_dragged.iter() {
            commands.entity(dragged_gate).remove::<Drag>();
        }
    }
}

// ############################# Connector ##############################################

pub enum ConnectorType {
    In,
    Out,
}

pub struct Connector {
    ctype: ConnectorType,
}

impl Connector {
    /// Create a new connector for a logic node.
    pub fn new(commands: &mut Commands, position: Vec3, radius: f32, ctype: ConnectorType) -> Entity {
        let circle = shapes::Circle {
            radius: radius,
            center: Vec2::new(0., 0.),
        };

        let connector = GeometryBuilder::build_as(
            &circle,
            ShapeColors::outlined(Color::TEAL, Color::BLACK),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(5.0),
            },
            Transform::from_xyz(position.x, position.y, position.z),
        );

        commands
            .spawn_bundle(connector)
            .insert(Connector { ctype: ctype })
            .insert(Interactable::new(Vec2::new(0., 0.), Vec2::new(radius * 2., radius * 2.), 
                                      CONNECTOR_GROUP))
            .insert(Selectable)
            .insert(Draggable { update: false })
            .id()
    }
}

/// Highlight a connector by increasing its radius when the mouse
/// hovers over it.
fn highlight_connector_system(
    commands: Commands,
    // We need all connectors the mouse hovers over.
    mut q_hover: Query<&mut Transform, (With<Hover>, With<Connector>)>,
    mut q2_hover: Query<&mut Transform, (Without<Hover>, With<Connector>)>,
) { 
    for (mut transform) in q_hover.iter_mut() {
        transform.scale = Vec3::new(1.2, 1.2, transform.scale.z);
    }

    for (mut transform) in q2_hover.iter_mut() {
        transform.scale = Vec3::new(1.0, 1.0, transform.scale.z);
    }
}

pub struct ConnectionLineIndicator;

fn drag_connector(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    mw: Res<MouseWorldPos>,
    q_dragged: Query<(Entity, &GlobalTransform), (With<Drag>, With<Connector>)>,
    q_conn_line: Query<Entity, With<ConnectionLineIndicator>>
) {
    use bevy_prototype_lyon::entity::ShapeBundle;

    if let Ok((entity, transform)) = q_dragged.single() {
        if mb.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Drag>();
            if let Ok(conn_line) = q_conn_line.single() {
                commands.entity(conn_line).despawn();

            }
        } else {
            let conn_entity = if let Ok(conn_line) = q_conn_line.single() {
                commands.entity(conn_line).remove_bundle::<ShapeBundle>();
                conn_line
            } else {
                commands.spawn().insert(ConnectionLineIndicator).id()  
            };

            let shape = shapes::Line(Vec2::new(transform.translation.x, transform.translation.y), 
                                     Vec2::new(mw.x, mw.y));

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

