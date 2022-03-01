#![feature(let_chains)]
#![feature(array_from_fn)]

use bevy::prelude::*;

mod bot;
mod draw;
mod map;
mod ui;
mod util;

use bot::BotData;
use bot::BotState;
use bot::edit::InstructionsEditor;
use draw::DrawTimer;
use map::BoxData;
use map::EntityKind;
use map::GridPos;
use map::Level;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    StartScreen,
    Programming,
    Running,
    ChangeLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    let levels = map::read_levels("./levels.json");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::StartScreen)
        .insert_resource(levels.clone())
        .insert_resource(levels.levels[0].clone())
        .insert_resource(bot::edit::InstructionsEditor::new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_system_set(SystemSet::on_enter(GameState::StartScreen).with_system(
            |mut state: ResMut<State<GameState>>| {
                state.set(GameState::ChangeLevel).unwrap();
            },
        ))
        .add_system_set(SystemSet::on_enter(GameState::ChangeLevel).with_system(
            |mut commands: Commands,
             level: Res<Level>,
             mut state: ResMut<State<GameState>>,
             mut instructions_editor: ResMut<InstructionsEditor>,
             map_tiles: Query<Entity, With<draw::MapTile>>,
             queryyy: Query<Entity, With<EntityKind>>| {
                 *instructions_editor = InstructionsEditor::new();
                 
                for e in map_tiles.iter() {
                    commands.entity(e).despawn();
                }
                for e in queryyy.iter() {
                    commands.entity(e).despawn();
                }

                for &bot_pos in &level.bots {
                    commands
                        .spawn()
                        .insert(bot::BotData::from_iter(bot_pos, Direction::Right, []))
                        .insert(bot::BotState::new(Direction::Right))
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
            },
        ))
        .add_system_set(
            SystemSet::on_enter(GameState::Programming)
                .with_system(ui::programming::init)
                .with_system(
                    (|world: &mut World| {
                        let mut with_pos = Vec::new();
                        for (entity, data) in world.query::<(Entity, &BotData)>().iter(world) {
                            with_pos.push((entity, data.start_position));
                        }
                        for (entity, data) in world.query::<(Entity, &BoxData)>().iter(world) {
                            with_pos.push((entity, data.start_position));
                        }
                        for (entity, data) in with_pos {
                            world.entity_mut(entity).insert(data);
                        }
                    })
                    .exclusive_system(),
                )
                .with_system(draw::init_map_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Programming)
                .with_system(ui::programming::update)
                .with_system(
                   |mut input: ResMut<Input<KeyCode>>,
                    mut game_state: ResMut<State<GameState>>| {
                        if input.pressed(KeyCode::Escape) {
                            input.release(KeyCode::Escape);
                            dbg!("hi");
                            game_state.set(GameState::ChangeLevel).unwrap();
                        }
                    }),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Programming)
                .with_system(util::delete_local_entities)
                .with_system(ui::programming::exit),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Running)
                .with_system(draw::init_map_system)
                .with_system(ui::running::init)
                .with_system(
                    |mut commands: Commands, mut query: Query<(Entity, &BotData)>| {
                        for (entity, _) in query.iter_mut() {
                            commands
                                .entity(entity)
                                .insert(BotState::new(Direction::Right));
                        }
                    },
                ),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running)
                .with_system(bot::progress_world)
                .with_system(draw::update_map_system)
                .with_system(ui::running::update),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Running)
                .with_system(util::delete_local_entities)
                .with_system(
                    |mut commands: Commands, query: Query<Entity, With<BotData>>| {
                        for entity in query.iter() {
                            commands.entity(entity).remove::<BotState>();
                        }
                    },
                ),
        )
        .add_system_set(SystemSet::on_update(GameState::Running))
        .run();
}
