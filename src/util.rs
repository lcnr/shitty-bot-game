use bevy::prelude::*;

use crate::{
    bot::{self, edit::InstructionsEditor, BotData, VoidedOrExited, BotState},
    map::{self, BoxData, EntityKind, Level, LevelList},
    CurrentLevel, GameState,
};

#[derive(Component)]
pub struct StateLocal;

pub fn delete_local_entities(mut commands: Commands, to_remove: Query<(Entity, &StateLocal)>) {
    for (entity, _local) in to_remove.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_level_data(
    mut level: ResMut<Level>,
    levels: Res<LevelList>,
    current: Res<CurrentLevel>,
) {
    *level = levels.levels[current.0].clone();
}

pub fn to_start(mut state: ResMut<State<GameState>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        state.set(GameState::StartScreen).unwrap();
    }
}

pub fn spawn_map_entities(
    mut commands: Commands,
    level: Res<Level>,
    mut state: ResMut<State<GameState>>,
    mut instructions_editor: ResMut<InstructionsEditor>,
    queryyy: Query<Entity, With<EntityKind>>,
) {
    *instructions_editor = InstructionsEditor::new();

    for e in queryyy.iter() {
        commands.entity(e).despawn();
    }

    for &(bot_pos, dir) in &level.bots {
        commands
            .spawn()
            .insert(bot::BotData::new(bot_pos, dir))
            .insert(map::EntityKind::Robot);
    }
    for &box_pos in &level.boxes {
        commands
            .spawn()
            .insert(map::BoxData {
                start_position: box_pos,
            })
            .insert(map::EntityKind::Box);
    }

    state.set(GameState::Programming).unwrap();
}

pub fn reset_bot_and_box_state(world: &mut World) {
    let mut with_pos = Vec::new();
    for (entity, data) in world.query::<(Entity, &BotData)>().iter(world) {
        with_pos.push((entity, data.start_position));
    }
    for (entity, data) in world.query::<(Entity, &BoxData)>().iter(world) {
        with_pos.push((entity, data.start_position));
    }
    for (entity, data) in with_pos {
        world
            .entity_mut(entity)
            .insert(data)
            .remove::<VoidedOrExited>();
        world.entity_mut(entity).remove::<BotState>();
    }
}
