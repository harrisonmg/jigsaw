use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
};
use game::{Cursor, CursorMovedEvent, PlayerConnectedEvent, Puzzle, Uuid};

use crate::states::AppState;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), player_cursors_setup)
            .add_systems(
                Update,
                player_connected_update.run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                player_cursors_update.run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct CursorComponent {}

#[derive(Bundle)]
pub struct CursorBundle {
    pub cursor: CursorComponent,

    #[bundle]
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

fn add_cursor(
    cursor: &Cursor,
    player_id: Uuid,
    puzzle: &Puzzle,
    meshes: &mut Assets<Mesh>,
    mut cursor_map: &mut CursorMap,
    mut commands: &mut Commands,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 1.0]; 3]);
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));

    //let entity = commands.spawn(CursorBundle {}).id();
    //cursor_map.0.insert(player_id, entity);
}

#[derive(Resource)]
pub struct CursorMap(HashMap<Uuid, Entity>);

fn player_cursors_setup(
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let mut map = CursorMap(HashMap::new());
    for (id, cursor) in puzzle.cursors().iter() {
        add_cursor(
            cursor,
            id.clone(),
            puzzle.as_ref(),
            meshes.as_mut(),
            &mut map,
            &mut commands,
        );
    }
    commands.insert_resource(map);
}

fn player_connected_update(
    mut player_connected_events: EventReader<PlayerConnectedEvent>,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cursor_map: ResMut<CursorMap>,
    mut commands: Commands,
) {
    for event in player_connected_events.iter() {
        add_cursor(
            &event.cursor,
            event.player_id,
            puzzle.as_ref(),
            meshes.as_mut(),
            cursor_map.as_mut(),
            &mut commands,
        );
    }
}

fn player_cursors_update(
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
