use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_asset_loader::{AssetLoader, AssetCollection};

mod node;

use node::*;
use nodus::world2d::{
    camera2d::Camera2DPlugin, 
    interaction2d::Interaction2DPlugin
};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
  AssetLoading,
  InGame,
}

fn main() {
    let mut app = App::build();

    AssetLoader::new(GameState::AssetLoading, GameState::InGame)
        .with_collection::<FontAssets>()
        .build(&mut app);

    app.add_state(GameState::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.41, 0.41, 0.41)))
        .insert_resource(WindowDescriptor {
            title: "nodus".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin) // immegiate 2d drawing
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(Camera2DPlugin)
        .add_plugin(Interaction2DPlugin)
        .add_plugin(NodeInGamePlugin)
        .run();
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub main: Handle<Font>,
}
