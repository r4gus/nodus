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
use nodus::world2d::{
    camera2d::Camera2DPlugin, 
    interaction2d::Interaction2DPlugin
};

const NODE_GROUP: u8 = 0;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.41, 0.41, 0.41)))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "nodus".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin) // immegiate 2d drawing
        .add_plugin(NodePlugin)
        .add_plugin(Camera2DPlugin)
        .add_plugin(Interaction2DPlugin)
        .run();
}



