use std::iter;

use crate::bot::BotData;
use crate::bot::Instruction;
use crate::util::StateLocal;
use bevy::prelude::*;

pub mod programming;
pub mod running;

mod buttons {
    use bevy::prelude::*;
    pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    pub const VALID_MEM: Color = Color::rgb(0.1, 0.1, 0.1);
    pub const INVALID_MEM: Color = Color::rgb(0.8, 0.3, 0.3);
    pub const SELECTED_MEM: Color = Color::rgb(0.1, 0.5, 0.1);
}
use buttons::*;

pub struct MemUi {
    user_names: [Entity; 32],
    user_values: [Entity; 32],
}

pub fn initialize_mem(mut commands: Commands, asset_server: Res<AssetServer>) {
    // mk ui
    let mut user_names = Vec::new();
    let mut user_values = Vec::new();
    for y in 0..32 / 4 {
        commands.spawn_bundle(TextBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                position_type: PositionType::Absolute,
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position: Rect {
                    left: Val::Auto,
                    right: Val::Percent(42.0),
                    top: Val::Percent(15.0 + y as f32 * 9.0),
                    bottom: Val::Auto,
                },
                ..Default::default()
            },
            text: Text::with_section(
                &format!("{}", y * 4),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 25.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                Default::default(),
            ),
            ..Default::default()
        });
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
                                top: Val::Percent(13.0 + y as f32 * 9.0),
                                bottom: Val::Auto,
                            },
                            ..Default::default()
                        },
                        color: VALID_MEM.into(),
                        ..Default::default()
                    })
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
                                top: Val::Percent(17.0 + y as f32 * 9.0),
                                bottom: Val::Auto,
                            },
                            ..Default::default()
                        },
                        color: VALID_MEM.into(),
                        ..Default::default()
                    })
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

pub fn refresh_mem(
    mem_ui: Res<MemUi>,
    mem: Query<&BotData>,
    mut color: Query<&mut UiColor>,
    children: Query<&Children>,
    mut text: Query<&mut Text>,
) {
    for mem in mem.iter() {
        let iter = iter::zip(
            &mem.instructions,
            iter::zip(&mem_ui.user_names, &mem_ui.user_values),
        );
        for (&instr, (&name, &value)) in iter {
            {
                let mut color = color.get_mut(name).unwrap();
                *color = VALID_MEM.into();
                let text_entity = children.get(name).unwrap()[0];
                text.get_mut(text_entity).unwrap().sections[0].value =
                    Instruction::from_repr(instr).map_or(String::new(), |i| i.to_string());
            }
            {
                let mut color = color.get_mut(value).unwrap();
                *color = VALID_MEM.into();
                let text_entity = children.get(value).unwrap()[0];
                text.get_mut(text_entity).unwrap().sections[0].value = instr.to_string();
            }
        }
    }
}

pub trait CornerButton: Sync + Send + 'static {
    const MK: fn(Entity) -> Self;
    const MSG: &'static str;
}

pub fn add_button<T: CornerButton>(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                    T::MSG,
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

    commands.insert_resource(T::MK(start_button));
}

pub fn remove_button<T: CornerButton>(mut commands: Commands) {
    commands.remove_resource::<T>();
}
