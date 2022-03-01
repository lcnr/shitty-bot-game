use std::iter;

use super::buttons::*;
use super::CornerButton;
use super::MemUi;
use crate::bot::edit::InstructionsEditor;
use crate::bot::BotData;
use crate::util::StateLocal;
use crate::GameState;
use bevy::prelude::*;

pub struct StartButton(Entity);
impl CornerButton for StartButton {
    const MK: fn(Entity) -> Self = StartButton;
    const MSG: &'static str = "Start";
}

pub fn update(
    mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<State<GameState>>,
    input: Res<Input<KeyCode>>,
    mut mem: ResMut<InstructionsEditor>,
    mut mem_ui: Res<MemUi>,
    mut start_button: Res<StartButton>,
    mut color: Query<&mut UiColor>,
    mut children: Query<&Children>,
    mut text: Query<&mut Text>,
) {
    let mut clicked_entity = None;
    for (entity, interaction) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => clicked_entity = Some(entity),
            _ => {}
        }
    }

    if let Ok((entity, interaction)) = interaction_query.get_mut(start_button.0) {
        let mut color = color.get_mut(entity).unwrap();
        match interaction {
            Interaction::Clicked => {
                state.set(GameState::Running).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }

    let mut update_cell = None;
    for (i, &ui) in mem_ui.user_names.iter().enumerate() {
        if Some(ui) == clicked_entity {
            update_cell = Some(Some((true, i)));
        }
    }
    for (i, &ui) in mem_ui.user_values.iter().enumerate() {
        if Some(ui) == clicked_entity {
            update_cell = Some(Some((false, i)));
        }
    }

    for input in input.get_just_pressed() {
        let cell = if let Some(cell) = mem.active_cell_data() {
            cell
        } else {
            match input {
                KeyCode::Tab => update_cell = Some(Some((true, 0))),
                _ => {}
            }
            break;
        };

        // TODO: numpad + tab support
        let c = match input {
            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',
            KeyCode::Key0 | KeyCode::Numpad0 => '0',
            KeyCode::Key1 | KeyCode::Numpad1 => '1',
            KeyCode::Key2 | KeyCode::Numpad2 => '2',
            KeyCode::Key3 | KeyCode::Numpad3 => '3',
            KeyCode::Key4 | KeyCode::Numpad4 => '4',
            KeyCode::Key5 | KeyCode::Numpad5 => '5',
            KeyCode::Key6 | KeyCode::Numpad6 => '6',
            KeyCode::Key7 | KeyCode::Numpad7 => '7',
            KeyCode::Key8 | KeyCode::Numpad8 => '8',
            KeyCode::Key9 | KeyCode::Numpad9 => '9',
            KeyCode::Space => ' ',
            KeyCode::Back => {
                cell.pop();
                continue;
            }
            KeyCode::Tab => {
                let cell_empty = cell.is_empty();
                if let Some((b, c)) = mem.active_cell {
                    update_cell = Some(Some(match (b, cell_empty) {
                        (true, true) => (false, c),
                        _ => (true, c + 1 % 32),
                    }));
                    break;
                } else {
                    continue;
                }
            }
            KeyCode::Return => {
                update_cell = Some(None);
                break;
            }
            key => {
                eprintln!("unexpected key: {:?}", key);
                continue;
            }
        };
        cell.push(c)
    }

    if let Some(value) = update_cell {
        if let Some((is_name, i)) = value {
            let id = if is_name {
                mem_ui.user_names[i]
            } else {
                mem_ui.user_values[i]
            };

            let mut color = color.get_mut(id).unwrap();
            *color = SELECTED_MEM.into();
        }

        if let Some((was_name, i)) = mem.on_selection_quit(value) {
            let id = if was_name {
                mem_ui.user_names[i]
            } else {
                mem_ui.user_values[i]
            };
            let mut color = color.get_mut(id).unwrap();
            if mem.error.is_none() {
                *color = VALID_MEM.into();
            } else {
                *color = INVALID_MEM.into();
            }
        }

        if let Some(s) = mem.active_cell_data() {
            s.clear();
        }
    }

    for (i, (data, &ui)) in iter::zip(&mem.user_names, &mem_ui.user_names).enumerate() {
        let text_entity = children.get(ui).unwrap()[0];
        text.get_mut(text_entity).unwrap().sections[0].value = data.clone();
    }
    for (i, (data, &ui)) in iter::zip(&mem.user_values, &mem_ui.user_values).enumerate() {
        let text_entity = children.get(ui).unwrap()[0];
        text.get_mut(text_entity).unwrap().sections[0].value = data.clone();
    }
}

pub fn exit(
    mut commands: Commands,
    mut instructions: ResMut<InstructionsEditor>,
    mut bot_data: Query<&mut BotData>,
) {
    // TODO: this is wrong, only one bot. move to update.
    instructions.on_selection_quit(None);
    for mut bot_data in bot_data.iter_mut() {
        bot_data.instructions = instructions.instructions;
    }
}
