use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::utils::HashMap;

use bevy::window::{CursorGrabMode, PrimaryWindow};
use game::{PieceMoveEvent, Puzzle};

mod piece;
use piece::{HeldPiece, PieceBundle, PieceComponent, PieceMap, PieceStack};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(setup)
        .add_event::<PieceMoveEvent>()
        .add_system(click_piece)
        .add_system(drag_piece)
        .add_system(move_piece)
        .add_system(sort_pieces)
        .run();
}

fn setup(mut commands: Commands, mut image_assets: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

    let puzzle = Puzzle::new(std::path::Path::new("../ymo.jpg"), 9);
    let mut piece_map = PieceMap(HashMap::new());
    let mut piece_stack = PieceStack(Vec::new());

    for (i, piece) in puzzle.pieces().enumerate() {
        let piece_entity = commands
            .spawn(PieceBundle::from_piece(&piece, &mut image_assets, i))
            .id();
        piece_map.0.insert(piece.index(), piece_entity);
        piece_stack.0.push(piece_entity);
    }

    commands.insert_resource(puzzle);
    commands.insert_resource(piece_map);
    commands.insert_resource(piece_stack);
}

fn click_piece(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    held_piece: Option<ResMut<HeldPiece>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    piece_query: Query<(&GlobalTransform, &PieceComponent)>,
    mut commands: Commands,
) {
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    if held_piece.is_none() {
                        let (camera, camera_transform) = camera_query.single();
                        let mut window = window_query.single_mut();
                        if let Some(click_pos) = window
                            .cursor_position()
                            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                            .map(|ray| ray.origin)
                        {
                            // prioritize highest z value (piece on top)
                            let mut candidate_index = None;
                            let mut candidate_z = f32::NEG_INFINITY;

                            for (piece_transform, piece) in piece_query.iter() {
                                let inverse_transform = Transform::from_matrix(
                                    piece_transform.compute_matrix().inverse(),
                                );
                                let relative_click_pos =
                                    inverse_transform.transform_point(click_pos);

                                let half_width = piece.width() / 2.0;
                                let half_height = piece.height() / 2.0;

                                let piece_z = piece_transform.translation().z;

                                if relative_click_pos.x.abs() <= half_width
                                    && relative_click_pos.y.abs() <= half_height
                                    && piece_z > candidate_z
                                {
                                    candidate_index = Some(piece.index());
                                    candidate_z = piece_z;
                                }
                            }

                            if let Some(index) = candidate_index {
                                commands.insert_resource(HeldPiece {
                                    index,
                                    cursor_position: click_pos.truncate(),
                                });

                                // grab cursor while holding piece to prevent moving far out of frame
                                window.cursor.grab_mode = CursorGrabMode::Confined;

                                break;
                            }
                        }
                    }
                }
                ButtonState::Released => {
                    if held_piece.is_some() {
                        commands.remove_resource::<HeldPiece>();
                        let mut window = window_query.single_mut();
                        window.cursor.grab_mode = CursorGrabMode::None;
                    }
                }
            }
        }
    }
}

fn drag_piece(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut piece_move_events: EventWriter<PieceMoveEvent>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    held_piece: Option<ResMut<HeldPiece>>,
    mut puzzle: ResMut<Puzzle>,
) {
    if let Some(mut held_piece) = held_piece {
        let (camera, camera_transform) = camera_query.single();
        for event in cursor_moved_events.iter() {
            let cursor_position = camera
                .viewport_to_world_2d(camera_transform, event.position)
                .unwrap();

            let cursor_delta = cursor_position - held_piece.cursor_position;
            piece_move_events.send_batch(puzzle.move_piece_rel(
                &held_piece.index,
                cursor_delta.x,
                cursor_delta.y,
            ));
            held_piece.cursor_position = cursor_position;
        }
    }
}

fn move_piece(
    mut piece_move_events: EventReader<PieceMoveEvent>,
    mut piece_query: Query<(&mut Transform, &mut PieceComponent)>,
    piece_map: Res<PieceMap>,
    mut piece_stack: ResMut<PieceStack>,
) {
    for event in piece_move_events.iter() {
        let piece_entity = piece_map.0.get(&event.index).unwrap().clone();
        let (mut transform, mut piece) = piece_query.get_mut(piece_entity).unwrap();
        transform.translation.x = event.x;
        transform.translation.y = event.y;
        transform.rotation = Quat::from_rotation_z(event.rotation);
    }
}

fn sort_pieces(
    mut piece_query: Query<(&mut Transform, &mut PieceComponent), With<PieceComponent>>,
    mut piece_stack: ResMut<PieceStack>,
) {
    let highest_piece_z = 900.0;
    let z_step = highest_piece_z / piece_stack.0.len() as f32;

    for (i, piece_entity) in piece_stack.0.iter().enumerate() {
        let (mut transform, mut piece) = piece_query.get_mut(piece_entity.clone()).unwrap();
        transform.translation.z = i as f32 * z_step;
        piece.stack_pos = i;
    }
}
