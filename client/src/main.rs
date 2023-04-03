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
use game::Puzzle;
use loader::LoaderPlugin;
use material::PieceMaterial;
use piece::PiecePlugin;
use states::AppState;
use viewport::{get_viewport_size, FullViewportPlugin};

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
        .add_systems(OnEnter(AppState::Setup), setup)
        .run();
}

fn setup(puzzle: Res<Puzzle>, mut commands: Commands) {
    let puzzle_size = Vec2::new(puzzle.width() as f32, puzzle.height() as f32);
    let small_side = puzzle_size.min_element();
    let min_zoom = 3.0 * small_side / Vec2::from(get_viewport_size());
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = min_zoom.min_element();
    commands.spawn(camera);
}
