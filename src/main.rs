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
use ui::programming::StartButton;
use ui::running::StopButton;

#[derive(Copy, Clone, Debug)]
pub struct CurrentLevel(usize);

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
    dbg!(&levels);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::StartScreen)
        .insert_resource(levels.clone())
        .insert_resource(levels.levels[0].clone())
        .insert_resource(CurrentLevel(0))
        .insert_resource(bot::edit::InstructionsEditor::new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_system_set(SystemSet::on_enter(GameState::StartScreen).with_system(
            |mut state: ResMut<State<GameState>>| {
                state.set(GameState::ChangeLevel).unwrap();
            },
        ))
        .add_system_set(
            SystemSet::on_enter(GameState::ChangeLevel).with_system(util::spawn_map_entities),
        )
        .add_system_set(SystemSet::on_exit(GameState::StartScreen).with_system(ui::initialize_mem))
        .add_system_set(
            SystemSet::on_enter(GameState::Programming)
                .with_system(ui::add_button::<StartButton>)
                .with_system(util::reset_bot_and_box_state.exclusive_system())
                .with_system(ui::refresh_mem)
                .with_system(ui::programming::init)
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
                            game_state.set(GameState::ChangeLevel).unwrap();
                        }
                    },
                ),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Programming)
                .with_system(ui::programming::exit.label("exit"))
                .with_system(util::delete_local_entities.after("exit"))
                .with_system(ui::remove_button::<StartButton>.after("exit")),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Running)
                .with_system(draw::init_timer.exclusive_system().before("update_map_sys"))
                .with_system(draw::init_map_system.before("update_map_sys"))
                .with_system(ui::refresh_mem)
                .with_system(ui::running::init)
                .with_system(ui::add_button::<StopButton>)
                .with_system(
                    |mut commands: Commands, mut query: Query<Entity, With<BotData>>| {
                        for entity in query.iter_mut() {
                            commands
                                .entity(entity)
                                .insert(BotState::new(Direction::Right));
                        }
                    },
                ).label("enter_running"),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running)
                .with_system(bot::progress_world.before("update_map_sys"))
                .with_system(draw::update_map_system.label("update_map_sys"))
                .with_system(ui::running::update1)
                .with_system(ui::refresh_mem.label("refresh"))
                .with_system(ui::running::update2.after("refresh"))
                .with_system(bot::level_complete_checker).after("enter_running"),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Running)
                .with_system(util::delete_local_entities)
                .with_system(ui::remove_button::<StopButton>)
                .with_system(|mut draw_steps: ResMut<draw::DrawUpdates>| {
                    draw_steps.data.clear();
                })
                .with_system(
                    |mut commands: Commands, query: Query<Entity, With<BotData>>| {
                        for entity in query.iter() {
                            commands.entity(entity).remove::<BotState>();
                        }
                    },
                ),
        )
        .run();
}
