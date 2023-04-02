use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

mod better_quad;
mod board;
mod colors;
mod cursor;
mod disable_context_menu;
mod loader;
mod material;
mod piece;
mod states;
mod viewport;

use board::BoardPlugin;
use cursor::WorldCursorPlugin;
use disable_context_menu::DisableContextMenuPlugin;
use loader::LoaderPlugin;
use material::PieceMaterial;
use piece::PiecePlugin;
use states::AppState;
use viewport::FullViewportPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_state::<AppState>()
        .add_plugin(FullViewportPlugin)
        .add_plugin(DisableContextMenuPlugin)
        .add_plugin(WorldCursorPlugin)
        .add_plugin(Material2dPlugin::<PieceMaterial>::default())
        .add_plugin(LoaderPlugin)
        .add_plugin(PiecePlugin)
        .add_plugin(BoardPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
