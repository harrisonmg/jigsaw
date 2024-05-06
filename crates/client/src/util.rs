use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn<T: QueryFilter>(entity_query: Query<Entity, T>, commands: &mut Commands) {
    if let Ok(entity) = entity_query.get_single() {
        if let Some(entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn_recursive();
        }
    }
}
