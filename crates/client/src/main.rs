use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

use game::Puzzle;

automod::dir!("src/");

use bevy_tweening::TweeningPlugin;
use board::BoardPlugin;
use cursor::MousePlugin;
use disable_context_menu::DisableContextMenuPlugin;
use help::HelpPlugin;
use material::PieceMaterial;
use network::NetworkPlugin;
use pieces::PiecePlugin;
use players::PlayersPlugin;
use states::AppState;
use viewport::get_viewport_size;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(DisableContextMenuPlugin)
        .add_plugin(MousePlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(Material2dPlugin::<PieceMaterial>::default())
        .add_plugin(NetworkPlugin)
        .add_plugin(PlayersPlugin)
        .add_plugin(PiecePlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(HelpPlugin)
        .add_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), setup)
        .add_systems(Update, center_camera.run_if(in_state(AppState::Playing)))
        .run();
}

fn setup(puzzle: Res<Puzzle>, mut commands: Commands) {
    let puzzle_size = Vec2::new(puzzle.width() as f32, puzzle.height() as f32);
    let small_side = puzzle_size.min_element();
    let initial_zoom = 3.0 * small_side / Vec2::from(get_viewport_size());
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = initial_zoom.min_element();
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
