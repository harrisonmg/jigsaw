use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::states::AppState;

const ZOOM_DENOM: f32 = 100.0;

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
    world_cursor_pos: ResMut<WorldCursorPosition>,
) {
    // TODO
    if scroll_events.iter().count() == 0 {
        return;
    }

    let mut camera_transform = camera_query.single_mut();
    let mut projection = projection_query.single_mut();
    let mut zoom_scale = projection.scale;

    let world_cursor_pos = world_cursor_pos.0.extend(0.0);
    let mut camera_cursor_offset = world_cursor_pos - camera_transform.translation;
    camera_cursor_offset.z = 0.0;
    camera_transform.translation = world_cursor_pos;
    camera_cursor_offset /= zoom_scale;

    zoom_scale = zoom_scale.ln();
    for event in scroll_events.iter() {
        zoom_scale -= event.y / ZOOM_DENOM;
    }
    zoom_scale = zoom_scale.exp();

    camera_cursor_offset *= zoom_scale;
    camera_transform.translation += camera_cursor_offset;
    projection.scale = zoom_scale;
}
