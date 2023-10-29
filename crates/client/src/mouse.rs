use std::time::Duration;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        ButtonState,
    },
    prelude::*,
};
use game::{
    PieceConnectionCheckEvent, PieceMovedEvent, PiecePickedUpEvent, PiecePutDownEvent, Puzzle,
};

use crate::{
    pieces::{HeldPiece, PieceComponent, PieceStack},
    states::AppState,
};

const ZOOM_FACTOR: f32 = 0.003;
const CLICK_TIME: Duration = Duration::from_millis(150);

#[derive(Resource, Debug)]
pub struct WorldCursorPosition(pub Vec2);

#[derive(Event)]
pub struct WorldCursorMoved(pub Vec2);

pub struct MousePlugin;

#[derive(Resource)]
pub struct MouseDown(pub bool);

#[derive(Resource)]
pub struct ClickTimer(pub Timer);

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldCursorMoved>()
            .insert_resource(WorldCursorPosition(Vec2::ZERO))
            .insert_resource(MouseDown(false))
            .insert_resource(ClickTimer(Timer::new(CLICK_TIME, TimerMode::Once)))
            .add_systems(PreUpdate, world_cursor.run_if(in_state(AppState::Playing)))
            .add_systems(
                PreUpdate,
                (pan, zoom)
                    .run_if(in_state(AppState::Playing))
                    .after(world_cursor),
            )
            .add_systems(
                Update,
                left_click.run_if(in_state(AppState::Playing)).after(pan),
            )
            .add_systems(
                Update,
                click_piece
                    .run_if(in_state(AppState::Playing))
                    .after(left_click),
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

fn left_click(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    time: Res<Time>,
    held_piece: Option<Res<HeldPiece>>,
    mut mouse_down: ResMut<MouseDown>,
    mut click_timer: ResMut<ClickTimer>,
) {
    click_timer.0.tick(time.delta());

    for event in mouse_button_events.iter() {
        if event.button != MouseButton::Left {
            continue;
        }

        match event.state {
            ButtonState::Pressed => {
                if click_timer.0.finished() && held_piece.is_none() {
                    mouse_down.0 = true;
                    click_timer.0.reset();
                }
            }
            ButtonState::Released => {
                if click_timer.0.finished() {
                    mouse_down.0 = false;
                }
            }
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
    mouse_down: Res<MouseDown>,
    mut piece_picked_up_events: EventWriter<PiecePickedUpEvent>,
    mut piece_put_down_events: EventWriter<PiecePutDownEvent>,
    mut piece_connection_check_events: EventWriter<PieceConnectionCheckEvent>,
    piece_query: Query<(&PieceComponent, &GlobalTransform, Entity)>,
    world_cursor_pos: Res<WorldCursorPosition>,
    held_piece: Option<ResMut<HeldPiece>>,
    puzzle: Res<Puzzle>,
    mut piece_stack: ResMut<PieceStack>,
    mut commands: Commands,
) {
    if mouse_down.is_changed() {
        match mouse_down.0 {
            true => {
                if held_piece.is_none() {
                    // prioritize highest z value (piece on top)
                    let mut candidate_entity = None;
                    let mut candidate_piece = None;
                    let mut candidate_z = f32::NEG_INFINITY;

                    for (piece, piece_transform, piece_entity) in piece_query.iter() {
                        let inverse_transform =
                            Transform::from_matrix(piece_transform.compute_matrix().inverse());
                        let relative_click_pos = inverse_transform
                            .transform_point(world_cursor_pos.0.extend(0.0))
                            .truncate();

                        let piece_z = piece_transform.translation().z;

                        if piece.within_sprite_bounds(relative_click_pos)
                            && puzzle.can_pick_up(&piece.index())
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
                        let candidate_piece = candidate_piece.unwrap();
                        piece_stack.put_on_top(piece_entity);
                        piece_picked_up_events.send(PiecePickedUpEvent {
                            player_id: None,
                            index: candidate_piece.index,
                        });
                        commands.insert_resource(candidate_piece);
                    }
                }
            }
            false => {
                if let Some(held_piece) = held_piece.as_deref() {
                    piece_put_down_events.send(PiecePutDownEvent {
                        player_id: None,
                        index: held_piece.index,
                    });
                    piece_connection_check_events.send(PieceConnectionCheckEvent {
                        index: held_piece.index,
                    });
                    commands.remove_resource::<HeldPiece>();
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
            let target = world_cursor.0 - held_piece.cursor_offset;
            piece_moved_events.send_batch(puzzle.try_move_piece(
                &held_piece.index,
                target.x,
                target.y,
            ));
        }
    }
}
