use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use nodus::world2d::camera2d::*;

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct BackgroundGrid;

pub fn draw_background_grid_system(
    mut commands: Commands,
    wnds: Res<Windows>,
    mut q: QuerySet<(
        QueryState<&mut Transform, (With<MainCamera>, Or<(Added<Transform>, Changed<Transform>)>)>,
        QueryState<(Entity, &mut Transform), With<BackgroundGrid>>,
    )>,
) {
    if let Ok(transform) = q.q0().get_single() {
        let x = transform.translation.x;
        let y = transform.translation.y;
        let x_start = (x as u32 & !0xFFF) as f32;
        let y_start = (y as u32 & !0xFFF) as f32;
        let scale = transform.scale.clone();

        if let Ok((entity, mut bgt)) = q.q1().get_single_mut() {
           bgt.translation.x = x_start; 
           bgt.translation.y = y_start;
        } else {
            let color = Color::rgba(0., 0., 0., 0.25);
            let window_size = get_primary_window_size(&wnds);
            let wx = window_size.x * 5.;
            let wy = window_size.y * 5.;


            let grid = commands
                .spawn()
                .insert(BackgroundGrid)
                .insert(Transform::from_xyz(x_start, y_start, 1.))
                .insert(GlobalTransform::from_xyz(x_start, y_start, 1.))
                .id();
            
            let mut evec = Vec::new();

            for xc in (0..wx as u32*4).step_by(120) {
                evec.push(commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Line(
                            Vec2::new(-wx*2. + xc as f32, wy*2.), 
                            Vec2::new(-wx*2. + xc as f32, -wy*2.)
                        ),
                        DrawMode::Stroke(StrokeMode::new(color, 7.0)),
                        Transform::from_xyz(0., 0., 1.),
                    )).id());
            }

            for yc in (0..wy as u32*4).step_by(120) {
                evec.push(commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Line(
                            Vec2::new(wx*2., -wy*2. + yc as f32), 
                            Vec2::new(-wx*2., -wy*2. + yc as f32)
                        ),
                        DrawMode::Stroke(StrokeMode::new(color, 7.0)),
                        Transform::from_xyz(0., 0., 1.),
                    )).id());
            }

            commands.entity(grid).push_children(&evec);
        }
    }
}
