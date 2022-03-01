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
use map::BoxData;
use map::GridPos;
use ui::programming::StartButton;
use ui::running::StopButton;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    StartScreen,
    Programming,
    Running,
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
        .insert_resource(levels.levels[0].map.clone())
        .insert_resource(bot::edit::InstructionsEditor::new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_system_set(SystemSet::on_enter(GameState::StartScreen).with_system(
            |mut state: ResMut<State<GameState>>| state.set(GameState::Programming).unwrap(),
        ))
        .add_system_set(
            SystemSet::on_enter(GameState::Programming)
                .with_system(ui::add_button::<StartButton>)
                .with_system(ui::initialize_mem)
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
            SystemSet::on_update(GameState::Programming).with_system(ui::programming::update),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Programming)
                .with_system(ui::programming::exit.label("exit"))
                .with_system(util::delete_local_entities.after("exit"))
                .with_system(ui::remove_button::<StartButton>.after("exit"))
                .with_system(ui::clear_mem.after("exit")),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Running)
                .with_system(draw::init_map_system)
                .with_system(ui::add_button::<StopButton>)
                .with_system(ui::initialize_mem)
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
                .with_system(ui::remove_button::<StopButton>)
                .with_system(ui::clear_mem)
                .with_system(
                    |mut commands: Commands, query: Query<Entity, With<BotData>>| {
                        for entity in query.iter() {
                            commands.entity(entity).remove::<BotState>();
                        }
                    },
                ),
        )
        .add_system_set(SystemSet::on_update(GameState::Running))
        .add_startup_system(|mut commands: Commands| {
            commands
                .spawn()
                .insert(bot::BotData::from_iter(
                    map::GridPos(3, 3),
                    Direction::Right,
                    [],
                ))
                .insert(bot::BotState::new(Direction::Right))
                .insert(map::EntityKind::Robot);
            commands
                .spawn()
                .insert(map::BoxData {
                    start_position: GridPos(4, 3),
                })
                .insert(map::EntityKind::Box);
        })
        .run();
}
