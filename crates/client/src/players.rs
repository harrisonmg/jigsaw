use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
};
use game::{Cursor, CursorMovedEvent, PlayerConnectedEvent, PlayerDisconnectedEvent, Puzzle, Uuid};

use crate::states::AppState;

const CURSOR_SIZE_RATIO: f32 = 0.4;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), player_cursors_setup)
            .add_systems(Update, player_connected.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                player_disconnected.run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, cursor_moved.run_if(in_state(AppState::Playing)));
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
            [cursor_size, 0.0, 0.0],
            [0.0, cursor_size, 0.0],
            [cursor_size, cursor_size, 0.0],
        ],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 1.0]; 3]);
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
    let mesh_handle = meshes.add(mesh);

    let material = materials.add(ColorMaterial::from(cursor.color));

    let bundle = CursorBundle {
        cursor: CursorComponent {},
        mesh_bundle: MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material,
            ..Default::default()
        },
    };

    let entity = commands.spawn(bundle).id();
    cursor_map.0.insert(player_id, entity);
}

#[derive(Resource)]
pub struct CursorMap(HashMap<Uuid, Entity>);

fn player_cursors_setup(
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let mut map = CursorMap(HashMap::new());
    for (id, cursor) in puzzle.cursors().iter() {
        add_cursor(
            cursor,
            id.clone(),
            puzzle.as_ref(),
            meshes.as_mut(),
            materials.as_mut(),
            &mut map,
            &mut commands,
        );
    }
    commands.insert_resource(map);
}

fn player_connected(
    mut player_connected_events: EventReader<PlayerConnectedEvent>,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cursor_map: ResMut<CursorMap>,
    mut commands: Commands,
) {
    for event in player_connected_events.iter() {
        add_cursor(
            &event.cursor,
            event.player_id,
            puzzle.as_ref(),
            meshes.as_mut(),
            materials.as_mut(),
            cursor_map.as_mut(),
            &mut commands,
        );
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
        }
        cursor_map.0.remove(&event.player_id);
    }
}

fn cursor_moved(
    mut cursor_moved_events: EventReader<CursorMovedEvent>,
    cursor_map: ResMut<CursorMap>,
    mut cursor_query: Query<&mut Transform, With<CursorComponent>>,
) {
    for event in cursor_moved_events.iter() {
        if let Some(player_id) = event.player_id {
            let entity = cursor_map.0.get(&player_id).unwrap();
            let mut transform = cursor_query.get_mut(*entity).unwrap();
            transform.translation.x = event.x;
            transform.translation.y = event.y;
        }
    }
}
