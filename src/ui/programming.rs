use std::iter;

use crate::bot::edit::InstructionsEditor;
use crate::bot::BotData;
use crate::util::StateLocal;
use crate::GameState;
use bevy::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const VALID_MEM: Color = Color::rgb(0.1, 0.1, 0.1);
const INVALID_MEM: Color = Color::rgb(0.8, 0.3, 0.3);
const SELECTED_MEM: Color = Color::rgb(0.1, 0.5, 0.1);

pub struct MemUi {
    user_names: [Entity; 32],
    user_values: [Entity; 32],
}

pub struct StartButton(Entity);

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    let start_button = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                position_type: PositionType::Absolute,
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position: Rect {
                    left: Val::Auto,
                    right: Val::Percent(1.0),
                    top: Val::Percent(1.0),
                    bottom: Val::Auto,
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(StateLocal)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Start",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    commands.insert_resource(StartButton(start_button));

    // mk ui
    let mut user_names = Vec::new();
    let mut user_values = Vec::new();
    for y in 0..32 / 4 {
        for x in 0..4 {
            user_names.push(
                commands
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(120.0), Val::Px(30.0)),
                            position_type: PositionType::Absolute,
                            margin: Rect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            position: Rect {
                                left: Val::Auto,
                                right: Val::Percent(32.0 - x as f32 * 10.0),
                                top: Val::Percent(15.0 + y as f32 * 10.0),
                                bottom: Val::Auto,
                            },
                            ..Default::default()
                        },
                        color: VALID_MEM.into(),
                        ..Default::default()
                    })
                    .insert(StateLocal)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 25.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    })
                    .id(),
            );
            user_values.push(
                commands
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(120.0), Val::Px(30.0)),
                            position_type: PositionType::Absolute,
                            margin: Rect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            position: Rect {
                                left: Val::Auto,
                                right: Val::Percent(32.0 - x as f32 * 10.0),
                                top: Val::Percent(19.0 + y as f32 * 10.0),
                                bottom: Val::Auto,
                            },
                            ..Default::default()
                        },
                        color: VALID_MEM.into(),
                        ..Default::default()
                    })
                    .insert(StateLocal)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 25.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    })
                    .id(),
            );
        }
    }

    commands.insert_resource(MemUi {
        user_names: user_names.try_into().unwrap(),
        user_values: user_values.try_into().unwrap(),
    })
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
        let mut color = color.get_mut(ui).unwrap();
        if Some(ui) == clicked_entity {
            *color = SELECTED_MEM.into();
            update_cell = Some(Some((true, i)));
        }
    }
    for (i, &ui) in mem_ui.user_values.iter().enumerate() {
        let mut color = color.get_mut(ui).unwrap();
        if Some(ui) == clicked_entity {
            *color = SELECTED_MEM.into();
            update_cell = Some(Some((false, i)));
        }
    }

    for input in input.get_just_pressed() {
        let cell = if let Some(cell) = mem.active_cell_data() {
            cell
        } else {
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
            KeyCode::Key0 => '0',
            KeyCode::Key1 => '1',
            KeyCode::Key2 => '2',
            KeyCode::Key3 => '3',
            KeyCode::Key4 => '4',
            KeyCode::Key5 => '5',
            KeyCode::Key6 => '6',
            KeyCode::Key7 => '7',
            KeyCode::Key8 => '8',
            KeyCode::Key9 => '9',
            KeyCode::Space => ' ',
            KeyCode::Back => {
                cell.pop();
                continue;
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
    mut instructions: Res<InstructionsEditor>,
    mut bot_data: Query<&mut BotData>,
) {
    // TODO: this is wrong, only one bot. move to update.
    for mut bot_data in bot_data.iter_mut() {
        bot_data.instructions = instructions.instructions;
    }

    commands.remove_resource::<MemUi>();
    commands.remove_resource::<StartButton>();
}
