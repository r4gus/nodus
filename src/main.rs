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
        .insert_resource(WindowDescriptor {
            title: "nodus".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin) // immegiate 2d drawing
        .add_plugin(NodePlugin)
        .add_plugin(Camera2DPlugin)
        .add_plugin(Interaction2DPlugin)
        .run();
}

pub struct GuiFonts {
    pub font1: Handle<Font>,
}

pub struct CursorTexture {
    pub default: Handle<ColorMaterial>,
    pub palm: Handle<ColorMaterial>,
    pub grab: Handle<ColorMaterial>,
}

pub struct Cursor;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    let assets = GuiFonts { font1: asset_server.load("fonts/FiraSans-Bold.ttf") };

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(50.0),
                    right: Val::Px(150.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "hello\nnodus!",
                TextStyle {
                    font: assets.font1.clone(),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            ..Default::default()
        });

    commands.insert_resource(assets); 
    commands.insert_resource(CursorTexture {
        default: asset_server.load("CursorSet/Win95Tri.png"),
        palm: asset_server.load("CursorSet/Win95DefPalm.png"),
        grab: asset_server.load("CursorSet/Win95DefGrab.png"),
    });

}
