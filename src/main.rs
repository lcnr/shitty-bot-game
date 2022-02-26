use bevy::prelude::*;

mod bot;
mod draw;
mod map;

#[derive(PartialEq, Eq)]
enum GameState {
    StartScreen,
    Programming,
    Running,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState::Programming)
        .insert_resource(map::Map::dummy_new())
        .insert_resource(draw::DrawUpdates::empty())
        .add_plugin(draw::DrawPlugin)
        .add_startup_system(|mut commands: Commands| {
            commands
                .spawn()
                .insert(map::GridPos(6, 3))
                .insert(map::EntityKind::Robot);
            commands
                .spawn()
                .insert(map::GridPos(4, 3))
                .insert(map::EntityKind::Box);
        })
        .add_system(bot::progress_world)
        .run();
}
