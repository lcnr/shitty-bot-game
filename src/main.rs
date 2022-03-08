#![feature(let_chains)]
#![feature(array_from_fn)]

use bevy::prelude::*;

mod bot;
mod draw;
mod level_loader;
mod map;
mod start;
mod ui;
mod util;

use bot::BotData;
use bot::BotState;
use serde::Deserialize;
use serde::Serialize;
use ui::programming::StartButton;
use ui::running::StopButton;

use crate::bot::ExecutionFailure;

#[derive(Copy, Clone, Debug)]
pub struct CurrentLevel(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    LoadScreen,
    StartScreen,
    Programming,
    Running,
    ChangeLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn start_up_system(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_asset::<map::LevelList>()
        .init_asset_loader::<level_loader::LevelListLoader>()
        .add_state(GameState::LoadScreen)
        .insert_resource(bot::edit::InstructionsEditor::new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_startup_system(level_loader::start_load_level_assets)
        .add_startup_system(start_up_system)
        //
        .add_system_set(
            SystemSet::on_update(GameState::LoadScreen)
                .with_system(level_loader::setup_level_resources),
        )
        //
        .add_system_set(
            SystemSet::on_enter(GameState::StartScreen)
                .with_system(ui::clear_mem.exclusive_system())
                .with_system(start::init),
        )
        .add_system_set(SystemSet::on_update(GameState::StartScreen).with_system(start::update))
        .add_system_set(
            SystemSet::on_exit(GameState::StartScreen)
                .with_system(util::delete_local_entities)
                .with_system(ui::init),
        )
        //
        .add_system_set(
            SystemSet::on_enter(GameState::ChangeLevel)
                .with_system(util::update_level_data.label("add_level"))
                .with_system(util::spawn_map_entities.after("add_level")),
        )
        //
        .add_system_set(
            SystemSet::on_enter(GameState::Programming)
                .with_system(ui::add_button::<StartButton>)
                .with_system(bot::init_state)
                .with_system(util::reset_bot_and_box_state.exclusive_system())
                .with_system(ui::refresh_mem)
                .with_system(draw::init_map_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Programming)
                .with_system(ui::programming::update)
                .with_system(util::to_start),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Programming)
                .with_system(ui::programming::exit.label("exit"))
                .with_system(util::delete_local_entities.after("exit"))
                .with_system(ui::remove_button::<StartButton>.after("exit")),
        )
        //
        .add_system_set(
            SystemSet::on_enter(GameState::Running)
                .with_system(draw::init_timer.exclusive_system())
                .with_system(draw::init_map_system)
                .with_system(ui::refresh_mem)
                .with_system(ui::running::init)
                .with_system(ui::add_button::<StopButton>),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running)
                .with_system(bot::progress_world.before("update_map_sys"))
                .with_system(draw::update_map_system.label("update_map_sys"))
                .with_system(ui::running::update1)
                .with_system(ui::refresh_mem.label("refresh"))
                .with_system(ui::running::update2.after("refresh"))
                .with_system(bot::level_complete_checker)
                .with_system(bot::failure_detector)
                .with_system(util::to_start),
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
                        commands.remove_resource::<ExecutionFailure>();
                    },
                ),
        )
        //
        .run();
}
