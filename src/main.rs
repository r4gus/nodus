use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_interact_2d::{
    InteractionPlugin,
    InteractionSource,
    drag::{DragPlugin},
    Group
};

mod node;

use node::*;
use nodus::world2d::World2DPlugin;

const NODE_GROUP: u8 = 0;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.9, 0.4, 0.4)))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "nodus".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(InteractionPlugin) // hover, click
        .add_plugin(DragPlugin) // drag n' drop
        .add_plugin(ShapePlugin) // immegiate 2d drawing
        .add_plugin(NodePlugin)
        .add_plugin(World2DPlugin)
        .run();
}



