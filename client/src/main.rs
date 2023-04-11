use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

mod animation;
mod better_quad;
mod board;
mod colors;
mod cursor;
mod disable_context_menu;
mod help;
mod loader;
mod material;
mod piece;
mod states;
mod viewport;

use bevy_tweening::TweeningPlugin;
use board::BoardPlugin;
use cursor::WorldCursorPlugin;
use disable_context_menu::DisableContextMenuPlugin;
use game::Puzzle;
use help::HelpPlugin;
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
        .add_plugin(TweeningPlugin)
        .add_plugin(Material2dPlugin::<PieceMaterial>::default())
        .add_plugin(LoaderPlugin)
        .add_plugin(PiecePlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(HelpPlugin)
        .add_systems(OnEnter(AppState::Setup), setup)
        .add_systems(Update, center_camera.run_if(in_state(AppState::Playing)))
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

fn center_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Space) {
        let mut transform = camera_query.get_single_mut().unwrap();
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}
