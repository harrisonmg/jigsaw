use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
};
use rand::Rng;

use game::{Cursor, PlayerCursorMovedEvent, PlayerDisconnectedEvent, Puzzle, Uuid};

use crate::states::AppState;
use crate::{
    mouse::{WorldCursorMoved, WorldCursorPosition},
    pieces::MAX_PIECE_HEIGHT,
};

const CURSOR_SIZE_RATIO: f32 = 0.6;
const CURSOR_HEIGHT: f32 = MAX_PIECE_HEIGHT + 1.0;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), player_cursors_setup)
            .add_systems(
                Update,
                player_cursor_moved.run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                player_disconnected.run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, mouse_moved.run_if(in_state(AppState::Playing)))
            .add_systems(Update, cursor_party.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Component)]
pub struct CursorComponent {}

#[derive(Bundle)]
pub struct CursorBundle {
    cursor: CursorComponent,
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

fn add_cursor(
    cursor: &Cursor,
    player_id: Uuid,
    puzzle: &Puzzle,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cursor_map: &mut CursorMap,
    commands: &mut Commands,
) {
    let cursor_size = puzzle.piece_width().min(puzzle.piece_height()) as f32 * CURSOR_SIZE_RATIO;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [0.0, -cursor_size, 0.0],
            [
                cursor_size * std::f32::consts::FRAC_1_SQRT_2,
                -cursor_size * std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            ],
        ],
    );
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
    let mesh_handle = meshes.add(mesh);

    let material = materials.add(ColorMaterial::from(cursor.color));

    let bundle = CursorBundle {
        cursor: CursorComponent {},
        mesh_bundle: MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material,
            transform: Transform::from_xyz(cursor.x, cursor.y, -1.0),
            ..Default::default()
        },
    };

    let entity = commands.spawn(bundle).id();
    cursor_map.0.insert(player_id, entity);
}

#[derive(Resource)]
pub struct CursorMap(HashMap<Uuid, Entity>);

#[derive(Resource)]
pub struct CursorColor(Color);

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let val: u32 = rng.gen_range(0..0xFFFFFF);
    Color::hex(format!("{val:06x}")).unwrap()
}

fn player_cursors_setup(mut commands: Commands) {
    commands.insert_resource(CursorMap(HashMap::new()));
    commands.insert_resource(CursorColor(random_color()));
}

fn player_cursor_moved(
    mut cursor_moved_events: EventReader<PlayerCursorMovedEvent>,
    mut cursor_map: ResMut<CursorMap>,
    mut cursor_query: Query<&mut Transform, With<CursorComponent>>,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    for event in cursor_moved_events.iter() {
        if let Some(player_id) = event.player_id {
            if let Some(entity) = cursor_map.0.get(&player_id) {
                if let Ok(mut transform) = cursor_query.get_mut(*entity) {
                    transform.translation.x = event.cursor.x;
                    transform.translation.y = event.cursor.y;
                    transform.translation.z = CURSOR_HEIGHT;
                }
            } else {
                add_cursor(
                    &event.cursor,
                    player_id,
                    puzzle.as_ref(),
                    meshes.as_mut(),
                    materials.as_mut(),
                    &mut cursor_map,
                    &mut commands,
                )
            }
        }
    }
}

fn player_disconnected(
    mut player_disconnected_events: EventReader<PlayerDisconnectedEvent>,
    mut cursor_map: ResMut<CursorMap>,
    mut commands: Commands,
) {
    for event in player_disconnected_events.iter() {
        if let Some(entity) = cursor_map.0.get(&event.player_id) {
            commands.get_entity(*entity).unwrap().despawn_recursive();
            cursor_map.0.remove(&event.player_id);
        }
    }
}

fn mouse_moved(
    world_cursor_moved_events: EventReader<WorldCursorMoved>,
    mut cursor_moved_events: EventWriter<PlayerCursorMovedEvent>,
    world_cursor_pos: Res<WorldCursorPosition>,
    cursor_color: Res<CursorColor>,
) {
    if !world_cursor_moved_events.is_empty() {
        cursor_moved_events.send(PlayerCursorMovedEvent {
            player_id: None,
            cursor: Cursor {
                color: cursor_color.0,
                x: world_cursor_pos.0.x,
                y: world_cursor_pos.0.y,
            },
        });
    }
}

fn cursor_party(
    cursor_query: Query<&Handle<ColorMaterial>, With<CursorComponent>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    puzzle: Res<Puzzle>,
) {
    if !puzzle.is_complete() {
        return;
    }

    let hue = time.elapsed_seconds() * 200.0 % 360.0;
    let new_color = Color::hsl(hue, 0.8, 0.5);
    for handle in cursor_query.iter() {
        if let Some(mut material) = materials.get_mut(handle) {
            material.color = new_color;
        }
    }
}
