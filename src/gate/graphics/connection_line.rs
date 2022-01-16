use crate::gate::core::{*, State};
use bevy::prelude::*;
use bevy_prototype_lyon::{
    prelude::*,
    entity::ShapeBundle,
};
use lyon_tessellation::path::path::Builder;
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable, Hover, Drag, Selected};
use nodus::world2d::camera2d::MouseWorldPos;

/// Sameple the cubic bezier curve, defined by s` (start),
/// `c1` (control point 1), `c2` (control point 2) and `e` (end),
/// at `t` (t e [0, 1]);
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
    
    const EPSILON: f32 = 16.;
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
            if dot <= EPSILON * EPSILON {
                return Some(t);
            }
        }
    }

    None
}

struct ConnectionLineShape<'a> {
    pub via: &'a [Vec2], 
}

impl<'a> Geometry for ConnectionLineShape<'a> {
    fn add_geometry(&self, b: &mut Builder) {
        let mut path = PathBuilder::new();
        path.move_to(self.via[0]);
        path.cubic_bezier_to(
            self.via[1],
            self.via[2],
            self.via[3],
        );

        b.concatenate(&[path.build().0.as_slice()]);
    }
}

pub struct LineResource {
    pub count: f32,
    pub timestep: f32,
    pub update: bool,
}

#[derive(Component)]
pub struct DataPoint {
    stepsize: f32,
    steps: f32,
}

#[derive(Component)]
pub struct LineHighLight;

pub fn draw_line_system(
    mut commands: Commands,
    mut q_line: Query<(Entity, &mut ConnectionLine, Option<&Children>, Option<&Selected>), ()>,
    q_transform: Query<(&Parent, &Connector, &GlobalTransform), ()>,
    q_outputs: Query<&Outputs, ()>,
    q_highlight: Query<Entity, With<LineHighLight>>,
    mut lr: ResMut<LineResource>,
    time: Res<Time>,
) {
    lr.count += time.delta_seconds();

    for (entity, mut conn_line, children, selected) in q_line.iter_mut() {
        if let Ok((t_parent, t_conn, t_from)) = q_transform.get(conn_line.output.entity) {
            // Set connection line color based on the value of the output.
            let color = if let Ok(outputs) = q_outputs.get(t_parent.0) {
                match outputs[t_conn.index] {
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
                        &ConnectionLineShape { via: &via },
                        DrawMode::Stroke(StrokeMode::new(color, 8.0)),
                        Transform::from_xyz(0., 0., 1.),
                    ));

                for e in q_highlight.iter() {
                    commands.entity(e).despawn();
                }
                
                // Highlight if selected.
                if let Some(_) = selected {
                    let child = commands.
                        spawn_bundle(
                            GeometryBuilder::build_as(
                                &ConnectionLineShape { via: &via },
                                DrawMode::Stroke(StrokeMode::new(
                                    Color::rgba(0.62, 0.79, 0.94, 0.5),
                                    18.0,
                                )),
                                Transform::from_xyz(0., 0., 0.),
                            )
                        )
                        .insert(LineHighLight).id();
                    
                    commands.entity(entity).add_child(child);
                } 

                conn_line.via = via;

                if color == Color::BLUE && lr.count >= lr.timestep {
                    let id = commands
                        .spawn_bundle(
                            GeometryBuilder::build_as(
                                &shapes::Circle {
                                    radius: 3.,
                                    center: Vec2::new(0., 0.),
                                },
                                DrawMode::Outlined {
                                    fill_mode: FillMode::color(Color::WHITE),
                                    outline_mode: StrokeMode::new(Color::WHITE, 1.0),
                                },
                                Transform::from_xyz(t_from.translation.x, t_from.translation.y, 3.),
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

pub fn line_selection_system(
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
                commands.entity(entity).insert(Selected);
                break;
            }
        }
    }
}

pub fn delete_line_system(
    input_keyboard: Res<Input<KeyCode>>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    q_line: Query<Entity, (With<Selected>, With<ConnectionLine>)>,
) {
    if input_keyboard.just_pressed(KeyCode::Delete) {
        for entity in q_line.iter() {
            ev_disconnect.send(DisconnectEvent {
                connection: entity,
                in_parent: None,
            });
        }
    }
}



pub fn draw_data_flow(
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
