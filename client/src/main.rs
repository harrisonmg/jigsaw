use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::utils::HashMap;

use bevy::window::{CursorGrabMode, PrimaryWindow};
use game::{PieceMoveEvent, Puzzle};

mod better_quad;
mod loader;
mod material;
mod piece;
mod states;

use loader::LoaderPlugin;
use material::PieceMaterial;
use piece::{HeldPiece, PieceBundle, PieceComponent, PieceMap, PieceStack};
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(Material2dPlugin::<PieceMaterial>::default())
        .add_systems(Update, bevy::window::close_on_esc)
        .add_state::<AppState>()
        .add_plugin(LoaderPlugin)
        .add_event::<PieceMoveEvent>()
        .add_systems(Update, setup.run_if(in_state(AppState::Setup)))
        .add_systems(Update, click_piece.run_if(in_state(AppState::Playing)))
        .add_systems(Update, drag_piece.run_if(in_state(AppState::Playing)))
        .add_systems(Update, move_piece.run_if(in_state(AppState::Playing)))
        .add_systems(Update, sort_pieces.run_if(in_state(AppState::Playing)))
        .add_systems(Update, zoom.run_if(in_state(AppState::Playing)))
        .run();
}

fn setup(
    mut commands: Commands,
    puzzle: Res<Puzzle>,
    mut image_assets: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PieceMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut piece_map = PieceMap(HashMap::new());
    let mut piece_stack = PieceStack(Vec::new());

    let mut i = 0;
    puzzle.with_pieces(|piece| {
        let piece_bundle =
            PieceBundle::new(piece, i, &mut image_assets, &mut meshes, &mut materials);
        let piece_entity = commands.spawn(piece_bundle).id();
        piece_map.0.insert(piece.index(), piece_entity);
        piece_stack.0.push(piece_entity);
        i += 1;
    });

    commands.insert_resource(piece_map);
    commands.insert_resource(piece_stack);
    next_state.set(AppState::Playing);
}

#[allow(clippy::too_many_arguments)]
fn click_piece(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut piece_move_events: EventWriter<PieceMoveEvent>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut piece_query: Query<(&GlobalTransform, &mut PieceComponent, Entity)>,
    mut puzzle: ResMut<Puzzle>,
    mut held_piece: Option<ResMut<HeldPiece>>,
    mut piece_stack: ResMut<PieceStack>,
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
                            let mut candidate_entity = None;
                            let mut candidate_z = f32::NEG_INFINITY;

                            for (piece_transform, _, piece_entity) in piece_query.iter() {
                                let inverse_transform = Transform::from_matrix(
                                    piece_transform.compute_matrix().inverse(),
                                );
                                let relative_click_pos =
                                    inverse_transform.transform_point(click_pos);

                                let half_width = puzzle.piece_width() as f32 / 2.0;
                                let half_height = puzzle.piece_height() as f32 / 2.0;

                                let piece_z = piece_transform.translation().z;

                                if relative_click_pos.x.abs() <= half_width
                                    && relative_click_pos.y.abs() <= half_height
                                    && piece_z > candidate_z
                                {
                                    candidate_entity = Some(piece_entity);
                                    candidate_z = piece_z;
                                }
                            }

                            if let Some(piece_entity) = candidate_entity {
                                let (_, mut piece, _) = piece_query.get_mut(piece_entity).unwrap();
                                commands.insert_resource(HeldPiece {
                                    index: piece.index(),
                                    cursor_position: click_pos.truncate(),
                                });

                                piece_stack.put_on_top(&mut piece, candidate_entity.unwrap());

                                // grab cursor while holding piece to prevent moving far out of frame
                                // TODO this doesn't work as expected in wasm
                                //window.cursor.grab_mode = CursorGrabMode::Confined;

                                break;
                            }
                        }
                    }
                }
                ButtonState::Released => {
                    if held_piece.is_some() {
                        let piece = held_piece.unwrap();
                        piece_move_events.send_batch(puzzle.make_group_connections(&piece.index));
                        held_piece = None;
                        commands.remove_resource::<HeldPiece>();
                        let mut window = window_query.single_mut();
                        // TODO
                        //window.cursor.grab_mode = CursorGrabMode::None;
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
                Transform::from_xyz(cursor_delta.x, cursor_delta.y, 0.0),
            ));
            held_piece.cursor_position = cursor_position;
        }
    }
}

#[allow(unused)]
fn spin_piece(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut piece_move_events: EventWriter<PieceMoveEvent>,
    keys: Res<Input<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    held_piece: Option<ResMut<HeldPiece>>,
    mut puzzle: ResMut<Puzzle>,
) {
    if let Some(mut held_piece) = held_piece {
        if keys.any_pressed([KeyCode::LShift, KeyCode::RShift]) {
            for event in mouse_motion_events.iter() {
                // This doesn't work for some reason
                let angle = event.delta.y / 10000.0;
                piece_move_events.send_batch(puzzle.move_piece_rel(
                    &held_piece.index,
                    Transform::from_rotation(Quat::from_rotation_z(angle)),
                ));

                let (camera, camera_transform) = camera_query.single();
                if let Some(cursor_position) = window_query
                    .get_single()
                    .unwrap()
                    .cursor_position()
                    .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
                {
                    held_piece.cursor_position = cursor_position;
                }
            }
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
        let piece_entity = *piece_map.0.get(&event.index).unwrap();
        let (mut transform, mut piece) = piece_query.get_mut(piece_entity).unwrap();
        transform.translation.x = event.x;
        transform.translation.y = event.y;
        transform.rotation = Quat::from_rotation_z(event.rotation);
        piece_stack.put_on_top(&mut piece, piece_entity);
    }
}

fn sort_pieces(
    mut piece_query: Query<(&mut Transform, &mut PieceComponent), With<PieceComponent>>,
    mut piece_stack: ResMut<PieceStack>,
) {
    let piece_count = piece_query.iter().len();
    let highest_piece_z = 900.0;
    let z_step = highest_piece_z / piece_count as f32;

    let mut stack_offset = 0;
    let mut i = 0;
    piece_stack.0.retain(|piece_entity| {
        let (mut transform, mut piece) = piece_query.get_mut(*piece_entity).unwrap();
        if piece.stack_pos == i {
            piece.stack_pos -= stack_offset;
            transform.translation.z = piece.stack_pos as f32 * z_step;
            i += 1;
            true
        } else {
            stack_offset += 1;
            i += 1;
            false
        }
    });
}

fn zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut projection_query: Query<&mut OrthographicProjection>,
) {
    let mut projection = projection_query.get_single_mut().unwrap();
    let mut zoom_scale = projection.scale.ln();
    for event in scroll_events.iter() {
        zoom_scale -= event.y / 10.0;
    }
    projection.scale = zoom_scale.exp();
}
