use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::*;

mod gate;
mod component;
mod radial_menu;

use component::*;
use nodus::world2d::{camera2d::Camera2DPlugin, interaction2d::Interaction2DPlugin};
use crate::gate::file_browser::*;

use radial_menu::RadialMenu;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    InGame,
}

fn main() {
    let mut app = App::new();

    AssetLoader::new(GameState::AssetLoading)
        .continue_to_state(GameState::InGame)
        .with_collection::<FontAssets>()
        .with_collection::<GateAssets>()
        .build(&mut app);

    app.add_state(GameState::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(0.75, 0.75, 0.75)))
        .insert_resource(WindowDescriptor {
            title: "nodus".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(EguiFileBrowserPlugin)
        .add_plugin(ShapePlugin) // 2d drawing
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(Camera2DPlugin)
        .add_plugin(Interaction2DPlugin)
        .add_plugin(LogicComponentSystem)
        .add_plugin(RadialMenu)
        .run();
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/hack.bold.ttf")]
    pub main: Handle<Font>,
}
