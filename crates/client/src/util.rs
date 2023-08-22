use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};

pub fn despawn<T: ReadOnlyWorldQuery>(entity_query: Query<Entity, T>, mut commands: Commands) {
    if let Ok(entity) = entity_query.get_single() {
        if let Some(entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn_recursive();
        }
    }
}
