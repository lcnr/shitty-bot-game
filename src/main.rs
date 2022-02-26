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

#[derive(Clone, Copy)]
enum Direction {
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
        .add_system(bot::progress_world)
        .run();
}
