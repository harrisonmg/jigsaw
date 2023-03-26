use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        ButtonState,
    },
    prelude::*,
};
use game::{PieceMoved, Puzzle};

use crate::{
    piece::{HeldPiece, PieceComponent, PieceStack},
    states::AppState,
};

const ZOOM_FACTOR: f32 = 0.003;

#[derive(Resource)]
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
    // TODO fix multi button mouse bug
    //let left = mouse_buttons.pressed(MouseButton::Left);
    //let middle = mouse_buttons.pressed(MouseButton::Middle);
    //let right = mouse_buttons.pressed(MouseButton::Right);
    //info!("{left}, {middle}, {right}");
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
    world_cursor_pos: ResMut<WorldCursorPosition>,
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
    mut piece_move_events: EventWriter<PieceMoved>,
    mut piece_query: Query<(&GlobalTransform, &mut PieceComponent, Entity)>,
    world_cursor_pos: Res<WorldCursorPosition>,
    mut puzzle: ResMut<Puzzle>,
    mut held_piece: Option<ResMut<HeldPiece>>,
    mut piece_stack: ResMut<PieceStack>,
    mut commands: Commands,
) {
    for event in mouse_button_events.iter() {
        // TODO
        //info!("{event:?}");
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    if held_piece.is_none() {
                        // prioritize highest z value (piece on top)
                        let mut candidate_entity = None;
                        let mut candidate_z = f32::NEG_INFINITY;

                        for (piece_transform, _, piece_entity) in piece_query.iter() {
                            let inverse_transform =
                                Transform::from_matrix(piece_transform.compute_matrix().inverse());
                            let relative_click_pos =
                                inverse_transform.transform_point(world_cursor_pos.0.extend(0.0));

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
                            commands.insert_resource(HeldPiece(piece.index()));
                            piece_stack.put_on_top(&mut piece, candidate_entity.unwrap());
                            break;
                        }
                    }
                }
                ButtonState::Released => {
                    if held_piece.is_some() {
                        let piece_index = held_piece.unwrap().0;
                        piece_move_events.send_batch(puzzle.make_group_connections(&piece_index));
                        held_piece = None;
                        commands.remove_resource::<HeldPiece>();
                    }
                }
            }
        }
    }
}

fn drag_piece(
    mut world_cursor_moved_events: EventReader<WorldCursorMoved>,
    mut piece_moved_events: EventWriter<PieceMoved>,
    held_piece: Option<ResMut<HeldPiece>>,
    mut puzzle: ResMut<Puzzle>,
) {
    if let Some(HeldPiece(piece_index)) = held_piece.as_deref() {
        for event in world_cursor_moved_events.iter() {
            piece_moved_events.send_batch(puzzle.move_piece_rel(
                &piece_index,
                Transform::from_translation(event.0.extend(0.0)),
            ));
        }
    }
}
