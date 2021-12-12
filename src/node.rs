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
            .add_event::<ConnectEvent>()
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
            .add_system(drag_gate_system.system())
            .add_system(drag_connector_system.system().label("drag_conn_system"))
            .add_system(connect_nodes.system().after("drag_conn_system"))
            .add_system(draw_line_system.system());
        
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

const GATE_SIZE: f32 = 100.;

struct GateSize {
    width: f32,
    height: f32,
    in_step: f32,
    out_step: f32,
    offset: f32,
}

impl Gate {

    fn get_distances(factor: f32, cin: f32, cout: f32) -> GateSize {
        let width = GATE_SIZE;
        let height = GATE_SIZE + if factor > 2. {
            (factor - 1.) * GATE_SIZE / 2.
        } else { 0. };
        let in_step = -(height / (cin + 1.));
        let out_step = -(height / (cout + 1.));
        let offset = height / 2.;

        GateSize {
            width,
            height,
            in_step,
            out_step,
            offset
        }
    }

    pub fn new(
        commands: &mut Commands, 
        name: String,
        x: f32, y: f32, 
        in_range: NodeRange, 
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>
    ) {
        static Z_INDEX: AtomicI32 = AtomicI32::new(1);
        
        let factor = if in_range.min >= out_range.min { in_range.min } else { out_range.min };
        let dists = Gate::get_distances(factor as f32, in_range.min as f32, out_range.min as f32);

        let zidx = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;
        let shape = shapes::Rectangle {
            width: dists.width,
            height: dists.height,
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
            .insert(Interactable::new(Vec2::new(0., 0.), Vec2::new(dists.width, dists.height), NODE_GROUP))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();
        
        let mut entvec: Vec<Entity> = Vec::new();
        for i in 1..=in_range.min {
            entvec.push(Connector::new(commands, 
                                       Vec3::new(-75., dists.offset + i as f32 * dists.in_step, zidx), 
                                       12., 
                                       ConnectorType::In,
                                       (i - 1) as usize));
        }

        commands.entity(parent).push_children(&entvec);
        entvec.clear();

        for i in 1..=out_range.min {
            entvec.push(Connector::new(commands, 
                                       Vec3::new(75., dists.offset + i as f32 * dists.out_step, zidx), 
                                       12., 
                                       ConnectorType::Out,
                                       (i - 1) as usize));
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

macro_rules! trans {
    ( $( $fun:expr ),* ) => {
        vec![ $( Box::new($fun) ),* ]
    };
    ( $( $fun:expr ),+ ,) => {
        trans![ $( $fun ),* ]
    };
}

fn setup(mut commands: Commands) {
    Gate::new(&mut commands, 
              "NOT Gate".to_string(), 
              0., 0., 
              NodeRange { min: 1, max: 1 },
              NodeRange { min: 1, max: 1 },
              trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
              },]
              );

    Gate::new(&mut commands, 
              "AND Gate".to_string(), 
              250., 0., 
              NodeRange { min: 2, max: 16 },
              NodeRange { min: 1, max: 1 },
              trans![|inputs| {
                  let mut ret = State::High;
                  for i in inputs {
                    match i {
                        State::None => { ret = State::None; },
                        State::Low => { ret = State::Low; break; },
                        State::High => { },
                    }
                  }
                  ret
              },]
            );
}


fn drag_gate_system(
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectorType {
    In,
    Out 
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
    pub fn new(commands: &mut Commands, position: Vec3, radius: f32, ctype: ConnectorType, index: usize) -> Entity {
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
            .insert(Connector { 
                ctype,
                index
            })
            .insert(Connections(Vec::new()))
            .insert(Free)
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
    use bevy_prototype_lyon::entity::ShapeBundle;

    if let Ok((entity, transform, connector)) = q_dragged.single() {
        // If the LMB is released we check if we can connect two connectors.
        if mb.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Drag>();

            // We dont need the visual connection line any more.
            // There will be another system responsible for
            // drawing the connections between nodes.
            if let Ok(conn_line) = q_conn_line.single() {
                commands.entity(conn_line).despawn();
            }

            // Try to connect input and output.
            if let Ok((drop_target, drop_connector)) = q_drop.single() {
                // One can only connect an input to an output.
                if connector.ctype != drop_connector.ctype {
                    // Send connection event.
                    match connector.ctype {
                        ConnectorType::In => {
                            ev_connect.send( 
                                ConnectEvent {
                                    output: drop_target,
                                    output_index: drop_connector.index,
                                    input: entity,
                                    input_index: connector.index
                                }
                            );
                        },
                        ConnectorType::Out => {
                            ev_connect.send(
                                ConnectEvent {
                                    output: entity,
                                    output_index: connector.index,
                                    input: drop_target,
                                    input_index: drop_connector.index,
                                }
                            );
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

struct ConnectEvent {
    output: Entity,
    output_index: usize,
    input: Entity,
    input_index: usize
}

/// Handle incomming connection events.
fn connect_nodes(
    mut commands: Commands,
    mut ev_connect: EventReader<ConnectEvent>,
    mut q_conns: Query<(&Parent, &mut Connections), ()>,
    mut q_parent: Query<&mut Targets>,
) {
    for ev in ev_connect.iter() {
        eprintln!("connect");
        let line = ConnectionLine::new(
            &mut commands,
            ConnInfo {
                entity: ev.output,
                index: ev.output_index            
            },
            ConnInfo {
                entity: ev.input,
                index: ev.input_index
            },
        );

        let input_parent = if let Ok((parent, mut connections)) = q_conns.get_mut(ev.input) {
            connections.0.push(line);
            parent.0
        } else { continue };
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
    pub fn new(commands: &mut Commands, output: ConnInfo, input: ConnInfo) -> Entity {
        commands
            .spawn()
            .insert(ConnectionLine {
                output,
                via: Vec::new(),
                input,
            }).id()
    }
}

fn draw_line_system(
    mut commands: Commands,
    q_line: Query<(Entity, &ConnectionLine), ()>,
    q_transform: Query<(&Parent, &Connector, &GlobalTransform), ()>,
    q_outputs: Query<&Outputs, ()>,
) {
    use bevy_prototype_lyon::entity::ShapeBundle;

    for (entity, conn_line) in q_line.iter() {
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
                // Remove old line
                commands.entity(entity).remove_bundle::<ShapeBundle>();

                // Insert new line
                let shape = shapes::Line(Vec2::new(t_from.translation.x, t_from.translation.y), 
                                         Vec2::new(t_to.translation.x, t_to.translation.y));

                let line = GeometryBuilder::build_as(
                    &shape,
                    ShapeColors::outlined(Color::TEAL, color),
                    DrawMode::Outlined {
                        fill_options: FillOptions::default(),
                        outline_options: StrokeOptions::default().with_line_width(10.0),
                    },
                    Transform::from_xyz(0., 0., 1.),
                );

                commands.entity(entity).insert_bundle(line);
            }
        }
   }
}
