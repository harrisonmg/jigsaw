use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::{log::LogPlugin, time::common_conditions::on_timer};

use game::{
    PieceConnectionEvent, PieceMovedEvent, PiecePickedUpEvent, PiecePutDownEvent,
    PlayerCursorMovedEvent, PlayerDisconnectedEvent, Puzzle,
};

automod::dir!("src/");

use board::BoardPlugin;
use cursors::CursorPlugin;
use disable_context_menu::DisableContextMenuPlugin;
use material::PieceMaterial;
use mouse::MousePlugin;
use network::NetworkPlugin;
use pieces::PiecePlugin;
use states::AppState;
use ui::UiPlugin;
use viewport::get_viewport_size;

fn main() {
    #[allow(unused_mut)]
    let mut log_plugin = LogPlugin::default();

    #[cfg(debug_assertions)]
    {
        log_plugin.level = bevy::log::Level::DEBUG;
        log_plugin.filter = "warn,client=debug,game=debug".into();
    }

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(log_plugin),
            DisableContextMenuPlugin,
            MousePlugin,
            Material2dPlugin::<PieceMaterial>::default(),
            NetworkPlugin,
            CursorPlugin,
            PiecePlugin,
            BoardPlugin,
            UiPlugin,
        ))
        .add_state::<AppState>()
        .add_event::<PieceMovedEvent>()
        .add_event::<PiecePickedUpEvent>()
        .add_event::<PiecePutDownEvent>()
        .add_event::<PieceConnectionEvent>()
        .add_event::<PlayerCursorMovedEvent>()
        .add_event::<PlayerDisconnectedEvent>()
        .add_systems(OnEnter(AppState::Loading), load)
        .add_systems(OnEnter(AppState::Setup), setup)
        .add_systems(Update, center_camera.run_if(in_state(AppState::Playing)))
        .insert_resource(PuzzleComplete(false))
        .add_systems(
            Update,
            puzzle_complete_check
                .run_if(in_state(AppState::Playing))
                .run_if(on_timer(Duration::from_millis(500))),
        )
        .run();
}

fn load(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup(puzzle: Res<Puzzle>, mut projection_query: Query<&mut OrthographicProjection>) {
    let puzzle_size = Vec2::new(puzzle.width() as f32, puzzle.height() as f32);
    let small_side = puzzle_size.min_element();
    let initial_zoom = 3.0 * small_side / Vec2::from(get_viewport_size());
    let mut proj = projection_query.get_single_mut().unwrap();
    proj.scale = initial_zoom.min_element();
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

#[derive(Resource)]
pub struct PuzzleComplete(pub bool);

fn puzzle_complete_check(puzzle: Res<Puzzle>, mut puzzle_complete: ResMut<PuzzleComplete>) {
    if !puzzle_complete.0 {
        puzzle_complete.0 = puzzle.is_complete();
    } else {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let location = document.location().unwrap();
        location.reload().unwrap();
    }
}
