use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;
use nodus::world2d::interaction::*;

pub struct NodePlugin;

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
            );
            //.add_system(interact_with_node.system())
            //.add_system(drag_node.system());
            

        
        info!("NodePlugin loaded");
    }
}

/*
fn interact_with_node(
    interaction_state: Res<InteractionState>,
    mut query: Query<(Entity, &mut ShapeColors), With<Gate>>
) {
    //info!("{}", interaction_state.get_group(Group(crate::NODE_GROUP)).len());
    for (entity, mut shape_color) in query.iter_mut() {
        if interaction_state
            .get_group(Group(crate::NODE_GROUP))
            .iter()
            .find(|(e, _)| *e == entity)
            .is_some()
        {
            info!("fuck");
            shape_color.main = Color::TEAL;
        } 
    }
}

fn drag_node(
  mut commands: Commands,
  mouse_button_input: Res<Input<MouseButton>>,
  interaction_state: Res<InteractionState>,
  dragged_node_query: Query<Entity, (With<Dragged>, With<Gate>)>,
) {
  // We're only interested in the release of the left mouse button
  if !mouse_button_input.just_released(MouseButton::Left) {
    return;
  }

  for dragged_node in dragged_node_query.iter() {
      info!("drgging {:?}", dragged_node);

      commands.entity(dragged_node).remove::<Dragged>();
  }
}
*/

fn move_test(mut query: Query<&mut Transform, With<Gate>>) {
    let mut transform = query.single_mut().unwrap();
    transform.translation.x = (transform.translation.x + 4.);
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

/// Flag for logic gates.
pub struct Gate;

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
        for i in 0..outputs.0.len() {
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
    add_gate(&mut commands, 0., 0., 100., 100.);
    add_gate(&mut commands, 50., 50., 100., 100.);
}

fn add_gate(commands: &mut Commands, x: f32, y: f32, width: f32, height: f32) {
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
        Transform::from_xyz(x, y, 1.),
    );

    let parent = commands
        .spawn_bundle(gate)
        .insert(Gate)
        .insert(Name(String::from("NOT Gate")))
        .insert(Inputs(vec![State::Low]))
        .insert(Outputs(vec![State::None]))
        .insert(Transitions(vec![Box::new(|inputs| { 
            match inputs[0] {
                State::None => State::None,
                State::High => State::Low,
                State::Low => State::High,
            }
        })]))
        .insert(Targets(vec![HashMap::new()]))
        .insert(Interactable::new(Vec2::new(x, y), Vec2::new(width, height)))
        .id();
    
    let child = add_connector(commands, 0., 0., 30., ConnectorType::In);

    commands.entity(parent).push_children(&[child]);
}

// ############################# Connector ##############################################

pub enum ConnectorType {
    In,
    Out,
}

pub struct Connector {
    ctype: ConnectorType,
}

fn add_connector(commands: &mut Commands, x: f32, y: f32, radius: f32, ctype: ConnectorType) -> Entity {
    let circle = shapes::Circle {
        radius: 30.,
        center: Vec2::new(x, y),
    };

    let connector = GeometryBuilder::build_as(
        &circle,
        ShapeColors::outlined(Color::TEAL, Color::BLACK),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(5.0),
        },
        Transform::from_xyz(x, y, 1.),
    );

    commands
        .spawn_bundle(connector)
        .insert(Connector { ctype: ctype })
        .id()
}

fn highlight_connector_system() { }



