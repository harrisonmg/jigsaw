use bevy::{prelude::*, utils::HashMap};
use rand::Rng;

use game::{Cursor, PlayerCursorMovedEvent, PlayerDisconnectedEvent, Uuid};

use crate::{
    mouse::{WorldCursorMoved, WorldCursorPosition},
    pieces::MAX_PIECE_HEIGHT,
};
use crate::{states::AppState, PuzzleComplete};

const CURSOR_SIZE: f32 = 0.6;
const CURSOR_HEIGHT: f32 = MAX_PIECE_HEIGHT + 1.0;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, cursors_setup)
            .add_systems(OnEnter(AppState::Playing), player_cursors_setup)
            .add_systems(
                Update,
                (player_disconnected, mouse_moved, cursor_party)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                PostUpdate,
                cursor_processing.run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                PostUpdate,
                player_cursor_moved
                    .run_if(in_state(AppState::Playing))
                    .after(cursor_processing),
            );
    }
}

#[derive(Component, Clone)]
pub struct CursorComponent {}

#[derive(Bundle, Clone)]
pub struct CursorBundle {
    pub cursor: CursorComponent,
    pub sprite_bundle: SpriteBundle,
}

#[derive(Resource)]
pub struct CursorPrefab(CursorBundle);

fn cursors_setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    // TODO sprite origin is wrong
    // TODO resource for both sprites
    // TODO sprite outline needs to be black
    let sprite_handle = asset_server.load("cursor/cursor.png");
    let bundle = CursorBundle {
        cursor: CursorComponent {},
        sprite_bundle: SpriteBundle {
            texture: sprite_handle,
            ..Default::default()
        },
    };
    commands.insert_resource(CursorPrefab(bundle));
}

#[derive(PartialEq, Eq, Hash)]
enum PlayerId {
    LocalPlayer,
    RemotePlayer(Uuid),
}

#[derive(Resource)]
pub struct CursorMap(HashMap<PlayerId, Entity>);

#[derive(Resource)]
pub struct CursorColor(Color);

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let val: u32 = rng.gen_range(0..0xFFFFFF);
    Color::hex(format!("{val:06x}")).unwrap()
}

fn player_cursors_setup(
    mut commands: Commands,
    cursor_query: Query<Entity, With<CursorComponent>>,
) {
    commands.insert_resource(CursorMap(HashMap::new()));
    commands.insert_resource(CursorColor(random_color()));

    for cursor_entity in cursor_query.iter() {
        commands
            .get_entity(cursor_entity)
            .unwrap()
            .despawn_recursive();
    }
}

fn player_cursor_moved(
    mut cursor_moved_events: EventReader<PlayerCursorMovedEvent>,
    mut cursor_map: ResMut<CursorMap>,
    mut cursor_query: Query<&mut Transform, With<CursorComponent>>,
    cursor_prefab: Res<CursorPrefab>,
    mut commands: Commands,
) {
    for event in cursor_moved_events.iter() {
        let new_translation = Vec3::new(event.cursor.x, event.cursor.y, CURSOR_HEIGHT);

        let player_id = match event.player_id {
            None => PlayerId::LocalPlayer,
            Some(uuid) => PlayerId::RemotePlayer(uuid),
        };

        if let Some(entity) = cursor_map.0.get(&player_id) {
            if let Ok(mut transform) = cursor_query.get_mut(*entity) {
                transform.translation = new_translation;
            }
        } else {
            let mut bundle = cursor_prefab.0.clone();
            bundle.sprite_bundle.transform.translation = new_translation;
            bundle.sprite_bundle.sprite.color = event.cursor.color;
            let entity = commands.spawn(bundle).id();
            cursor_map.0.insert(player_id, entity);
        }
    }
}

fn player_disconnected(
    mut player_disconnected_events: EventReader<PlayerDisconnectedEvent>,
    mut cursor_map: ResMut<CursorMap>,
    mut commands: Commands,
) {
    for event in player_disconnected_events.iter() {
        let player_id = PlayerId::RemotePlayer(event.player_id);
        if let Some(entity) = cursor_map.0.get(&player_id) {
            commands.get_entity(*entity).unwrap().despawn_recursive();
            cursor_map.0.remove(&player_id);
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
    mut cursor_query: Query<&mut Sprite, With<CursorComponent>>,
    time: Res<Time>,
    puzzle_complete: Res<PuzzleComplete>,
) {
    if !puzzle_complete.0 {
        return;
    }

    let hue = time.elapsed_seconds() * 200.0 % 360.0;
    let new_color = Color::hsl(hue, 0.8, 0.5);
    for mut sprite in cursor_query.iter_mut() {
        sprite.color = new_color;
    }
}

fn cursor_processing() {}
