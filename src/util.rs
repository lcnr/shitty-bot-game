use bevy::prelude::*;

#[derive(Component)]
pub struct StateLocal;

pub fn delete_local_entities(mut commands: Commands, to_remove: Query<(Entity, &StateLocal)>) {
    for (entity, _local) in to_remove.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
