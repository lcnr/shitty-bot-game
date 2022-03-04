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

pub struct ErrorText(Entity);

const NO_ERROR: Color = Color::rgba(0.6, 0.7, 0.6, 0.5);
const ERROR: Color = Color::rgba(0.8, 0.4, 0.4, 0.7);

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let error_button = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                position_type: PositionType::Absolute,
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position: Rect {
                    left: Val::Percent(1.0),
                    right: Val::Percent(1.0),
                    top: Val::Percent(85.0),
                    bottom: Val::Percent(1.0),
                },
                ..Default::default()
            },
            color: NO_ERROR.into(),
            ..Default::default()
        })
        .insert(StateLocal)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    commands.insert_resource(ErrorText(error_button));
}

pub fn update(
    mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<State<GameState>>,
    input: Res<Input<KeyCode>>,
    mut mem: ResMut<InstructionsEditor>,
    mem_ui: Res<MemUi>,
    error_text: Res<ErrorText>,
    start_button: Res<StartButton>,
    mut color: Query<&mut UiColor>,
    children: Query<&Children>,
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

    if let Some(mut value) = update_cell {
        if value == mem.active_cell {
            value = None;
        }

        if let Some((was_name, i)) = mem.on_selection_quit(value) {
            if mem.error.is_none() {
                let mut c = color.get_mut(mem_ui.user_names[i]).unwrap();
                *c = VALID_MEM.into();
                drop(c);
                let mut c = color.get_mut(mem_ui.user_values[i]).unwrap();
                *c = VALID_MEM.into();
            } else {
                let mut color = color
                    .get_mut(if was_name {
                        mem_ui.user_names[i]
                    } else {
                        mem_ui.user_values[i]
                    })
                    .unwrap();
                *color = INVALID_MEM.into();
            }

            let mut error_color = color.get_mut(error_text.0).unwrap();
            let text_entity = children.get(error_text.0).unwrap()[0];
            let text = &mut text.get_mut(text_entity).unwrap().sections[0].value;
            if let Some(ref err) = mem.error {
                *error_color = ERROR.into();
                *text = err.clone();
            } else {
                *error_color = NO_ERROR.into();
                *text = String::new();
            }
        }

        if let Some((is_name, i)) = value {
            let id = if is_name {
                mem_ui.user_names[i]
            } else {
                mem_ui.user_values[i]
            };

            let mut color = color.get_mut(id).unwrap();
            *color = SELECTED_MEM.into();
        }

        if let Some(s) = mem.active_cell_data() {
            s.clear();
        }
    }

    for (data, &ui) in iter::zip(&mem.user_names, &mem_ui.user_names) {
        let text_entity = children.get(ui).unwrap()[0];
        text.get_mut(text_entity).unwrap().sections[0].value = data.clone();
    }
    for (data, &ui) in iter::zip(&mem.user_values, &mem_ui.user_values) {
        let text_entity = children.get(ui).unwrap()[0];
        text.get_mut(text_entity).unwrap().sections[0].value = data.clone();
    }
}

pub fn exit(
    mut commands: Commands,
    mut instructions: ResMut<InstructionsEditor>,
    mut bot_data: Query<&mut BotData>,
) {
    commands.remove_resource::<ErrorText>();
    // TODO: this is wrong, only one bot. move to update.
    instructions.on_selection_quit(None);
    for mut bot_data in bot_data.iter_mut() {
        bot_data.instructions = instructions.instructions;
    }
}
