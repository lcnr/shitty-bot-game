#![feature(let_chains)]
#![feature(array_from_fn)]

use bevy::prelude::*;

mod bot;
mod draw;
mod map;
mod ui;

use bot::Instruction;
use bot::Memory;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    // StartScreen,
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Programming)
        .insert_resource(map::Map::dummy_new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_system_set(
            SystemSet::on_enter(GameState::Programming).with_system(draw::init_map_system)
            .with_system(ui::programming_ui),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running)
                .with_system(bot::progress_world)
                .with_system(draw::update_map_system),
        )
        .add_startup_system(|mut commands: Commands| {
            commands
                .spawn()
                .insert(map::GridPos(3, 3))
                .insert(bot::BotData::from_iter(
                    map::GridPos(3, 3),
                    Direction::Right,
                    [
                        Memory::Instruction(Instruction::Walk),
                        Memory::Data(5),
                        Memory::Instruction(Instruction::TurnLeft),
                        Memory::Instruction(Instruction::Goto),
                        Memory::Data(0),
                    ],
                ))
                .insert(bot::BotState::new(Direction::Right))
                .insert(map::EntityKind::Robot);
            commands
                .spawn()
                .insert(map::GridPos(4, 3))
                .insert(map::EntityKind::Box);
        })
        .run();
}
