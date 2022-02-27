use bevy::prelude::*;

use crate::GameState;

pub fn programming_ui(mut state: ResMut<State<GameState>>) {
    state.set(GameState::Running).unwrap();
}