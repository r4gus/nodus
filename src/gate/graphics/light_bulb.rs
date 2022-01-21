use crate::gate::{
    core::{*, State},
    graphics::connector::*,
};
use crate::gate::serialize::*;
use super::*;
use nodus::world2d::interaction2d::{Interactable, Selectable, Draggable};
use std::sync::atomic::Ordering;
use bevy::prelude::*;
use bevy_prototype_lyon::{
    prelude::*,
    entity::ShapeBundle,
    shapes::SvgPathShape,
};

/// SVG path of a light bulb.
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

/// Light bulb component.
///
/// A light bulb is meant as a visual representation of a state.
/// It has one input connector to receive a signal from a connected
/// gate, input control or other logic component.
///
/// It glows if the input is [`State::High`].
#[derive(Component)]
pub struct LightBulb {
    state: State,
}

impl LightBulb {
    fn shape_bundle(color: Color) -> ShapeBundle {
        GeometryBuilder::build_as(
            &SvgPathShape {
                svg_doc_size_in_px: Vec2::new(580.922, 580.922),
                svg_path_string: LIGHT_BULB_PATH.to_string(),
            },
             DrawMode::Outlined {
                fill_mode: FillMode::color(color),
                outline_mode: StrokeMode::new(Color::BLACK, 8.0),
            },
            Transform::from_scale(Vec3::new(0.22, 0.22, 0.22)),
        )
    }

    /// Create a new light bulb at the specified position.
    pub fn spawn(
        commands: &mut Commands,
        position: Vec2,
    ) {
        let z = Z_INDEX.fetch_add(1, Ordering::Relaxed) as f32;

        let parent = commands
            .spawn()
            .insert(Transform::from_xyz(position.x, position.y, z))
            .insert(GlobalTransform::from_xyz(position.x, position.y, z))
            .insert(LightBulb { state: State::None })
            .insert(Name("Light Bulb".to_string()))
            .insert(Inputs(vec![State::None]))
            .insert(NodeType::LightBulb)
            .insert(Interactable::new(
                Vec2::new(0., 0.),
                Vec2::new(GATE_SIZE, GATE_SIZE),
                1,
            ))
            .insert(Selectable)
            .insert(Draggable { update: true })
            .id();

        let bulb = commands
            .spawn_bundle(LightBulb::shape_bundle(Color::WHITE))
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
}

pub fn light_bulb_system(
    mut commands: Commands,
    mut q_light: Query<(&Children, &Inputs, &mut LightBulb)>,
    mut draw: Query<&mut DrawMode, Without<Connector>>,
) {
    for (children, inputs, mut light) in q_light.iter_mut() {

        // Update the light bulbs visuals only if the state has changed.
        if inputs[0] != light.state {
            
            // Colorize the light bulb based on its new state.
            let color = match inputs[0] {
                State::High => Color::BLUE,
                _ => Color::WHITE,
            };
            
            // One of the entities children is the actual svg image. Find
            // and update its color;
            for &child in children.iter() {
                if let Ok(mut mode) = draw.get_mut(child) {
                    if let DrawMode::Outlined {
                        ref mut fill_mode,
                        ref mut outline_mode,
                    } = *mode {
                        fill_mode.color = color;
                    }
                }
            }

            light.state = inputs[0];
        }
    }
}
