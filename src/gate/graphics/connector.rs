use crate::gate::core::{*, State};
use bevy::prelude::*;
use bevy_prototype_lyon::{
    prelude::*,
    entity::ShapeBundle,
};
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable, Hover, Drag};
use std::sync::atomic::Ordering;
use nodus::world2d::camera2d::MouseWorldPos;

impl Connector {
    /// Create a new connector for a logic node.
    pub fn with_shape(
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
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::BLACK, 5.0),
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
                2,
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
        let id = Connector::with_shape(commands, position, radius, ctype, index);
        let line = shapes::Line(Vec2::new(-position.x, 0.), Vec2::new(0., 0.));
        let line_conn = GeometryBuilder::build_as(
            &line,
            DrawMode::Stroke(StrokeMode::new(Color::BLACK, 6.0)),
            Transform::from_xyz(0., 0., -1.),
        );

        let line_id = commands.spawn_bundle(line_conn).id();
        commands.entity(id).push_children(&[line_id]);
        id
    }
}

/// Highlight a connector by increasing its radius when the mouse
/// hovers over it.
pub fn highlight_connector_system(
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
#[derive(Debug, Clone, PartialEq, Component)]
pub struct ConnectionLineIndicator;

pub fn drag_connector_system(
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
    if let Ok((entity, transform, connector)) = q_dragged.get_single() {
        // If the LMB is released we check if we can connect two connectors.
        if mb.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Drag>();

            // We dont need the visual connection line any more.
            // There will be another system responsible for
            // drawing the connections between nodes.
            if let Ok(conn_line) = q_conn_line.get_single() {
                commands.entity(conn_line).despawn_recursive();
            }

            // Try to connect input and output.
            if let Ok((drop_target, drop_connector)) = q_drop.get_single() {
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
            let conn_entity = if let Ok(conn_line) = q_conn_line.get_single() {
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
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::WHITE),
                    outline_mode: StrokeMode::new(Color::BLACK, 10.0),
                },
                Transform::from_xyz(0., 0., 1.),
            );

            commands.entity(conn_entity).insert_bundle(line);
        }
    }
}
