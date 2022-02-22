use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

macro_rules! trans {
    ( $( $fun:expr ),* ) => {
        vec![ $( Box::new($fun) ),* ]
    };
    ( $( $fun:expr ),+ ,) => {
        trans![ $( $fun ),* ]
    };
}

pub(crate) use trans;

/// The name of an entity.
#[derive(Debug, Clone, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Name(pub String);

/// The input and output states of a logic gate.
///
/// # States
/// `None` - The state is unknown, for example because the gate
/// doesn't get a value for each input.
/// `High` - The sate is high (`1`).
/// `Low` - The state is low (`0`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Reflect,
    bevy::reflect::FromReflect,
    Serialize,
    Deserialize,
)]
pub enum State {
    None,
    High,
    Low,
}

impl Default for State {
    fn default() -> Self {
        Self::None
    }
}

/// Specify the minimum and maximum number a connectors for a logic component.
#[derive(Debug, Copy, Clone, PartialEq, Reflect, Default)]
pub struct NodeRange {
    pub min: u32,
    pub max: u32,
}

/// Gate ECS component.
///
/// # Attributes
///
/// * `inputs` - Current number of input connectors.
/// * `outputs` - Current number of output connectors.
/// * `in_range` - Allowed minimum and maximum of inputs connectors.
/// * `out_range` - Allowed minimum and maximum of inputs connectors.
#[derive(Debug, Clone, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Gate {
    pub inputs: u32,
    pub outputs: u32,
    pub in_range: NodeRange,
    pub out_range: NodeRange,
}

impl Gate {
    #[allow(dead_code)]
    pub fn new(
        commands: &mut Commands,
        name: &str,
        in_range: NodeRange,
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>,
    ) -> Entity {
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
            .insert(Targets(vec![
                TargetMap::from(HashMap::new());
                out_range.min as usize
            ]))
            .id();

        let mut connectors = Vec::new();
        for i in 0..in_range.min as usize {
            connectors.push(Connector::new(commands, ConnectorType::In, i));
        }
        for i in 0..in_range.min as usize {
            connectors.push(Connector::new(commands, ConnectorType::Out, i));
        }
        commands.entity(gate).push_children(&connectors);

        gate
    }

    #[allow(dead_code)]
    pub fn from_world(
        world: &mut World,
        name: &str,
        in_range: NodeRange,
        out_range: NodeRange,
        functions: Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>,
    ) -> Entity {
        let gate = world
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
            .insert(Targets(vec![
                TargetMap::from(HashMap::new());
                out_range.min as usize
            ]))
            .id();

        let mut connectors = Vec::new();
        for i in 0..in_range.min as usize {
            connectors.push(Connector::from_world(world, ConnectorType::In, i));
        }
        for i in 0..in_range.min as usize {
            connectors.push(Connector::from_world(world, ConnectorType::Out, i));
        }
        world.entity_mut(gate).push_children(&connectors);

        gate
    }
}

/// Input values of a gate.
#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct Inputs(pub Vec<State>);

impl Default for Inputs {
    fn default() -> Self {
        Inputs(Vec::new())
    }
}

impl Deref for Inputs {
    type Target = Vec<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Output values of a gate.
#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct Outputs(pub Vec<State>);

impl Default for Outputs {
    fn default() -> Self {
        Outputs(Vec::new())
    }
}

impl Deref for Outputs {
    type Target = Vec<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Outputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A set of transition functions `f: Inputs -> State`.
///
/// For a gate there should be a transition function for each output, that
/// calculates a new output value from the given inputs.
#[derive(Component)]
pub struct Transitions(pub Vec<Box<dyn Fn(&[State]) -> State + Send + Sync>>);

impl Default for Transitions {
    fn default() -> Self {
        Transitions(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Reflect, Default, Deserialize, Serialize)]
pub struct TIndex(pub Vec<usize>);

impl Deref for TIndex {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<usize>> for TIndex {
    fn from(v: Vec<usize>) -> Self {
        Self(v)
    }
}

/// Type that maps form a logc component (gate, input control, ...) to a set
/// of inputs, specified by a index.
///
/// The reason behind this is that the output of a logic component
/// can be connected to multiple inputs of another logic component.
/// This map is meant to keep track of all inputs of logic
/// components a output is connected to.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TargetMap(pub HashMap<Entity, TIndex>);

impl Deref for TargetMap {
    type Target = HashMap<Entity, TIndex>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TargetMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashMap<Entity, TIndex>> for TargetMap {
    fn from(map: HashMap<Entity, TIndex>) -> Self {
        Self(map)
    }
}

/// A vector that maps from outputs to connected nodes.
///
/// For a logic node, e.g. a gate, there should be a vector entry for
/// each output.
#[derive(Debug, Clone, PartialEq, Component, Deserialize, Serialize)]
pub struct Targets(pub Vec<TargetMap>);

impl Default for Targets {
    fn default() -> Self {
        Targets(Vec::new())
    }
}

impl Deref for Targets {
    type Target = Vec<TargetMap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Targets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Type of a connector.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum ConnectorType {
    In,
    Out,
}

/// A connector acts as the interface of a logic component,
/// e.g. logic gate.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct Connector {
    /// The type of the connector.
    pub ctype: ConnectorType,
    /// Its index in the context of a logical node.
    /// The index of a connector, with a certain connection
    /// type, must be uniq in context of a logic component.
    pub index: usize,
}

impl Connector {
    pub fn new(commands: &mut Commands, ctype: ConnectorType, index: usize) -> Entity {
        commands
            .spawn()
            .insert(Connector { ctype, index })
            .insert(Connections(Vec::new()))
            .insert(Free)
            .id()
    }

    pub fn from_world(world: &mut World, ctype: ConnectorType, index: usize) -> Entity {
        world
            .spawn()
            .insert(Connector { ctype, index })
            .insert(Connections(Vec::new()))
            .insert(Free)
            .id()
    }
}

/// Connection lines connected to this connector.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct Connections(pub Vec<Entity>);

impl Deref for Connections {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Connections {
    /// Return an iterator over all connections.
    fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

/// Marker component for free connectors.
///
/// Output connectors are always free, i.e.
/// one can connect them to multiple inputs.
#[derive(Debug, Copy, Clone, PartialEq, Hash, Component)]
pub struct Free;

/// Associate a index with a entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnInfo {
    pub entity: Entity,
    pub index: usize,
}

/// A connection between a output and a input.
///
/// The `via` vector can be used to store path coordinates
/// between two gates which can be helpful when visualizing
/// the connection.
#[derive(Debug, Clone, PartialEq, Component)]
pub struct ConnectionLine {
    pub output: ConnInfo,
    pub via: Vec<Vec2>,
    pub input: ConnInfo,
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
    /// The output is a vector with a start point, two control points and
    /// a end point.
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

/// System for calculating the state of each output using the corresponding
/// transition functions.
pub fn transition_system(mut query: Query<(&Inputs, &Transitions, &mut Outputs)>) {
    for (inputs, transitions, mut outputs) in query.iter_mut() {
        for i in 0..transitions.0.len() {
            outputs[i] = transitions.0[i](inputs);
        }
    }
}

/// System for writing the calculated output states to the inputs of each connected node.
pub fn propagation_system(
    from_query: Query<(&Outputs, &Targets)>,
    mut to_query: Query<&mut Inputs>,
) {
    for (outputs, targets) in from_query.iter() {
        for i in 0..outputs.len() {
            for (&entity, idxvec) in targets[i].iter() {
                if let Ok(mut inputs) = to_query.get_mut(entity) {
                    for &j in idxvec.iter() {
                        if j < inputs.len() {
                            inputs[j] = outputs[i];
                        }
                    }
                } else {
                    error!("Could not query inputs of given entity: {:?}", entity);
                }
            }
        }
    }
}

/// Event that asks the [`connect_event_system`] to connect
/// the specified `output` to the given `input`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectEvent {
    pub output: Entity,
    pub output_index: usize,
    pub input: Entity,
    pub input_index: usize,
    pub signal_success: bool, // Should signal success via NewConnectionEstablishedEvent.
}

/// A new connection has been established maybe somebody
/// wants to know.
#[derive(Debug, Clone, PartialEq)]
pub struct NewConnectionEstablishedEvent {
    pub id: Entity,
}

/// Handle incomming connection events.
///
/// This system handles incomming [`ConnectEvent`] events,
/// connecting the output one entity to the input of another.
pub fn connect_event_system(
    mut commands: Commands,
    mut ev_connect: EventReader<ConnectEvent>,
    mut ev_est: EventWriter<NewConnectionEstablishedEvent>,
    mut q_conns: Query<(&Parent, &mut Connections), ()>,
    mut q_parent: Query<&mut Targets>,
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
                // The points are not relevant for now and
                // can be updated later on.
                Vec3::new(0., 0., 0.),
                Vec3::new(0., 0., 0.),
            ),
        );

        // Add the new connection line to the set of lines already connected to the gate.
        let input_parent = if let Ok((parent, mut connections)) = q_conns.get_mut(ev.input) {
            connections.0.push(line);
            parent.0
        } else {
            continue;
        };

        // From this moment on the input connector isn't free
        // any more, up to the point where the connection is
        // removed.
        commands.entity(ev.input).remove::<Free>();

        // Also update the output connector.
        if let Ok((parent, mut connections)) = q_conns.get_mut(ev.output) {
            connections.0.push(line);

            // The target map hast to point to the input connector,
            // so it can receive updates.
            if let Ok(mut targets) = q_parent.get_mut(parent.0) {
                targets[ev.output_index]
                    .entry(input_parent)
                    .or_insert(TIndex::from(Vec::new()))
                    .push(ev.input_index);
            }
        }
        
        // Hey everybody, a new connection has been established!
        if ev.signal_success {
            ev_est.send(NewConnectionEstablishedEvent { id: line });
        }
    }
}

/// Request to the [`disconnect_event_system`] to
/// disconnect the given connection.
#[derive(Debug, Clone, PartialEq)]
pub struct DisconnectEvent {
    pub connection: Entity,
    pub in_parent: Option<Entity>,
}

/// Handle disconnect requests issued via a [`DisconnectEvent`].
pub fn disconnect_event_system(
    mut commands: Commands,
    mut ev_disconnect: EventReader<DisconnectEvent>,
    q_line: Query<&ConnectionLine>,
    mut q_conn: Query<(&Parent, Entity, &mut Connections)>,
    mut q_parent: Query<&mut Targets>,
    mut q_input: Query<&mut Inputs>,
) {
    for ev in ev_disconnect.iter() {
        // Try to fetch the connection line.
        if let Ok(line) = q_line.get(ev.connection) {
            let in_parent: Option<Entity>;

            // Unlink input connector (right hand side)
            if let Ok((parent_in, entity_in, mut connections_in)) =
                q_conn.get_mut(line.input.entity)
            {
                in_parent = Some(parent_in.0);

                // Reset input state of the given connector.
                if let Ok(mut inputs) = q_input.get_mut(parent_in.0) {
                    inputs[line.input.index] = State::None;
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
                    let size = targets[line.output.index]
                        .get(&parent)
                        .expect("Should have associated entry")
                        .len();

                    if size > 1 {
                        if let Some(index) = targets[line.output.index]
                            .get_mut(&parent)
                            .expect("Should have associated entry")
                            .iter()
                            .position(|x| *x == line.input.index)
                        {
                            targets[line.output.index]
                                .get_mut(&parent)
                                .expect("Should have associated entry")
                                .remove(index);
                        }
                    } else {
                        targets[line.output.index].remove(&parent);
                    }
                }
            }

            // Finally remove the connection line itself.
            commands.entity(ev.connection).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{State, *};
    use bevy::ecs::event::Events;

    #[test]
    fn test_connect() {
        // Setup world
        let mut world = World::default();

        // First stage for event handling
        let mut first_stage = SystemStage::parallel();
        first_stage.add_system(Events::<ConnectEvent>::update_system);
        first_stage.add_system(Events::<DisconnectEvent>::update_system);

        // Setup event resources
        world.insert_resource(Events::<ConnectEvent>::default());
        world.insert_resource(Events::<DisconnectEvent>::default());

        // Setup stage with our systems
        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(disconnect_event_system.system());
        update_stage.add_system(transition_system.system().label("transition"));
        update_stage.add_system(propagation_system.system().after("transition"));
        update_stage.add_system(connect_event_system.system());

        let not_gate1 = Gate::from_world(
            &mut world,
            "NOT Gate",
            NodeRange { min: 1, max: 1 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
            },],
        );

        let not_gate2 = Gate::from_world(
            &mut world,
            "NOT Gate",
            NodeRange { min: 1, max: 1 },
            NodeRange { min: 1, max: 1 },
            trans![|inputs| {
                match inputs[0] {
                    State::None => State::None,
                    State::Low => State::High,
                    State::High => State::Low,
                }
            },],
        );

        // Nothing should happen
        first_stage.run(&mut world);
        update_stage.run(&mut world);
        assert_eq!(
            world.entity(not_gate1).get::<Inputs>().unwrap()[0],
            State::None
        );
        assert_eq!(
            world.entity(not_gate1).get::<Outputs>().unwrap()[0],
            State::None
        );
        assert_eq!(
            world.entity(not_gate2).get::<Inputs>().unwrap()[0],
            State::None
        );
        assert_eq!(
            world.entity(not_gate2).get::<Outputs>().unwrap()[0],
            State::None
        );

        // Set input of gate 1 to low  -> should update the output of gate 1
        world.entity_mut(not_gate1).get_mut::<Inputs>().unwrap()[0] = State::Low;

        first_stage.run(&mut world);
        update_stage.run(&mut world);
        assert_eq!(
            world.entity(not_gate1).get::<Inputs>().unwrap()[0],
            State::Low
        );
        assert_eq!(
            world.entity(not_gate1).get::<Outputs>().unwrap()[0],
            State::High
        );
        assert_eq!(
            world.entity(not_gate2).get::<Inputs>().unwrap()[0],
            State::None
        );
        assert_eq!(
            world.entity(not_gate2).get::<Outputs>().unwrap()[0],
            State::None
        );

        // Now lets send a connection event
        /*
        world.get_resource_mut::<Events::<ConnectEvent>>().send(
            ConnectEvent {
                output:
            }
        );
        */
    }
}
