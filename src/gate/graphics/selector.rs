use bevy::prelude::*;
use crate::gate::{
    core::{Gate},
    graphics::{
        light_bulb::LightBulb,
        toggle_switch::ToggleSwitch,
        clk::Clk,
    },
};
use bevy_prototype_lyon::{
    prelude::*,
    render::Shape,
};
use nodus::world2d::{Lock, InteractionMode};
use nodus::world2d::camera2d::MouseWorldPos;
use nodus::world2d::interaction2d::{Hover, Selected};

#[derive(Debug, Clone, PartialEq, Component)]
pub struct SelectBox {
    start: Vec2,
}

pub fn selector_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    mw: Res<MouseWorldPos>,
    lock: Res<Lock>,
    mode: Res<InteractionMode>,
    q_gate: Query<(Entity, &Transform), (Or<(With<Gate>, With<LightBulb>, With<ToggleSwitch>, With<Clk>)>)>,
    q_hover: Query<Entity, (With<Hover>)>,
    mut q_select: Query<(Entity, &mut Path, &SelectBox), With<SelectBox>>,
) {
    if lock.0 || *mode != InteractionMode::Select { return; }

    if q_hover.is_empty() && mb.just_pressed(MouseButton::Left) {
        let frame = GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(1.0, 1.0),
                origin: RectangleOrigin::TopLeft,
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::rgba(0.72, 0.277, 0.0, 0.5)),
                outline_mode: StrokeMode::new(Color::rgba(0.72, 0.277, 0.0, 1.0), 2.0),
            },
            Transform::from_xyz(mw.x, mw.y, 900.0),
        );

        commands
            .spawn_bundle(frame)
            .insert(SelectBox { start: Vec2::new(mw.x, mw.y) });
    } else if !q_select.is_empty() && mb.just_released(MouseButton::Left) {
        if let Ok((entity, _, sb)) = q_select.get_single() {
            let (wx, wy) = if sb.start.x < mw.x {
                (sb.start.x, mw.x)
            } else {
                (mw.x, sb.start.x)
            };
            let (hx, hy) = if sb.start.y < mw.y {
                (sb.start.y, mw.y)
            } else {
                (mw.y, sb.start.y)
            };

            for (entity, transform) in q_gate.iter() {
                let (x, y) = (transform.translation.x, transform.translation.y);

                if wx <= x && x <= wy && hx <= y && y <= hy {
                    commands.entity(entity).insert(Selected);
                }
            }

            commands.entity(entity).despawn_recursive();
        }
    } else {
        if let Ok((_, mut path, sb)) = q_select.get_single_mut() {
            let w = mw.x - sb.start.x;
            let h = -(mw.y - sb.start.y);

            let s = &shapes::Rectangle {
                extents: Vec2::new(w, h),
                origin: RectangleOrigin::TopLeft,
            };

            *path = ShapePath::build_as(s);
        }
    }
}
