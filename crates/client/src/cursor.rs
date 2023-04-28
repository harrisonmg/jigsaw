use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        ButtonState,
    },
    prelude::*,
};
use bevy_tweening::Animator;
use game::{PieceMovedEvent, Puzzle};

use crate::{
    animation::{grow, shrink},
    pieces::{HeldPiece, PieceComponent, PieceMap, PieceStack},
    states::AppState,
};

const ZOOM_FACTOR: f32 = 0.003;

#[derive(Resource, Debug)]
pub struct WorldCursorPosition(pub Vec2);

pub struct WorldCursorMoved(pub Vec2);

pub struct WorldCursorPlugin;

impl Plugin for WorldCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldCursorMoved>()
            .insert_resource(WorldCursorPosition(Vec2::ZERO))
            .add_systems(Update, world_cursor.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                pan.run_if(in_state(AppState::Playing)).after(world_cursor),
            )
            .add_systems(
                Update,
                zoom.run_if(in_state(AppState::Playing)).after(world_cursor),
            )
            .add_systems(
                Update,
                click_piece.run_if(in_state(AppState::Playing)).after(pan),
            )
            .add_systems(
                Update,
                drag_piece
                    .run_if(in_state(AppState::Playing))
                    .after(click_piece),
            );
    }
}

fn world_cursor(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut world_cursor_moved_events: EventWriter<WorldCursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world_cursor_pos: ResMut<WorldCursorPosition>,
) {
    for event in cursor_moved_events.iter() {
        let (camera, camera_transform) = camera_query.single();
        if let Some(new_world_pos) = camera.viewport_to_world_2d(camera_transform, event.position) {
            let world_cursor_delta = new_world_pos - world_cursor_pos.0;
            world_cursor_moved_events.send(WorldCursorMoved(world_cursor_delta));

            world_cursor_pos.0 = new_world_pos;
        }
    }
}

fn pan(
    mut world_cursor_moved_events: EventReader<WorldCursorMoved>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut world_cursor_pos: ResMut<WorldCursorPosition>,
) {
    if mouse_buttons.any_pressed([MouseButton::Right, MouseButton::Middle]) {
        let mut camera_transform = camera_query.single_mut();
        for event in world_cursor_moved_events.iter() {
            camera_transform.translation -= event.0.extend(0.0);
            world_cursor_pos.0 -= event.0;
        }
    }
}

fn zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut projection_query: Query<&mut OrthographicProjection>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    world_cursor_pos: Res<WorldCursorPosition>,
) {
    let mut camera_transform = camera_query.single_mut();
    let mut projection = projection_query.single_mut();
    for event in scroll_events.iter() {
        let mut total_factor = 1.0 + event.y.abs() * ZOOM_FACTOR;
        if event.y > 0.0 {
            total_factor = 1.0 / total_factor;
        }
        projection.scale *= total_factor;
        let mut camera_cursor_offset = world_cursor_pos.0 - camera_transform.translation.truncate();
        camera_cursor_offset *= total_factor - 1.0;
        camera_transform.translation -= camera_cursor_offset.extend(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn click_piece(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut piece_query: Query<(
        &PieceComponent,
        &GlobalTransform,
        Entity,
        &mut Animator<Transform>,
    )>,
    world_cursor_pos: Res<WorldCursorPosition>,
    piece_map: Res<PieceMap>,
    held_piece: Option<ResMut<HeldPiece>>,
    puzzle: ResMut<Puzzle>,
    mut piece_stack: ResMut<PieceStack>,
    mut commands: Commands,
) {
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    if held_piece.is_none() {
                        // prioritize highest z value (piece on top)
                        let mut candidate_entity = None;
                        let mut candidate_piece = None;
                        let mut candidate_z = f32::NEG_INFINITY;

                        for (piece, piece_transform, piece_entity, _) in piece_query.iter() {
                            let inverse_transform =
                                Transform::from_matrix(piece_transform.compute_matrix().inverse());
                            let relative_click_pos = inverse_transform
                                .transform_point(world_cursor_pos.0.extend(0.0))
                                .truncate();

                            let piece_z = piece_transform.translation().z;

                            if piece.within_sprite_bounds(relative_click_pos)
                                && !puzzle.piece_group_locked(&piece.index())
                                && piece_z > candidate_z
                            {
                                candidate_entity = Some(piece_entity);
                                candidate_piece = Some(HeldPiece {
                                    index: piece.index(),
                                    cursor_offset: relative_click_pos,
                                });
                                candidate_z = piece_z;
                            }
                        }

                        if let Some(piece_entity) = candidate_entity {
                            piece_stack.put_on_top(piece_entity);

                            let (_, _, _, mut animator) =
                                piece_query.get_mut(piece_entity).unwrap();
                            grow(&mut animator);

                            commands.insert_resource(candidate_piece.unwrap());
                            break;
                        }
                    }
                }
                ButtonState::Released => {
                    if let Some(held_piece) = held_piece.as_deref() {
                        // TODO trigger server connection check
                        //piece_move_events
                        //    .send_batch(puzzle.make_group_connections(&held_piece.index));

                        let piece_entity = *piece_map.0.get(&held_piece.index).unwrap();
                        let (_, _, _, mut animator) = piece_query.get_mut(piece_entity).unwrap();
                        shrink(&mut animator);

                        commands.remove_resource::<HeldPiece>();
                    }
                }
            }
        }
    }
}

fn drag_piece(
    mut piece_moved_events: EventWriter<PieceMovedEvent>,
    held_piece: Option<ResMut<HeldPiece>>,
    mouse_buttons: Res<Input<MouseButton>>,
    world_cursor: Res<WorldCursorPosition>,
    mut puzzle: ResMut<Puzzle>,
) {
    if let Some(held_piece) = held_piece.as_deref() {
        if !mouse_buttons.any_pressed([MouseButton::Right, MouseButton::Middle]) {
            let piece_z = puzzle
                .piece(&held_piece.index)
                .unwrap()
                .transform()
                .translation
                .z;
            let piece_pos = Transform::from_translation(
                (world_cursor.0 - held_piece.cursor_offset).extend(piece_z),
            );
            piece_moved_events.send_batch(puzzle.try_move_piece(&held_piece.index, piece_pos));
        }
    }
}
