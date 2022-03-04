use crate::map::Level;
use crate::map::LevelList;
use crate::util::StateLocal;
use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct LevelId(usize);

const NOT_DONE: Color = Color::rgb(0.2, 0.5, 0.2);
const DONE: Color = Color::rgb(0.4, 1.0, 0.4);

pub fn init(mut commands: Commands, levels: Res<LevelList>, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(DirectionalLightBundle {
            transform: Transform::from_xyz(0.7, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        })
        .insert(StateLocal);
    let viewing_pos = Vec3::new(0.0, 0.0, 1.0);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(viewing_pos).looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        })
        .insert(StateLocal);

    for (i, &beaten) in levels.beaten.iter().enumerate() {
        commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Auto),
                    position_type: PositionType::Absolute,
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position: Rect {
                        left: Val::Percent(20.0 + (i % 6) as f32 * 10.0),
                        right: Val::Percent(75.0 - (i % 6) as f32 * 10.0),
                        top: Val::Percent(30.0 + (i / 6) as f32 * 10.0),
                        bottom: Val::Percent(65.0 + (i / 6) as f32 * 10.0),
                    },
                    ..Default::default()
                },
                color: if beaten { DONE.into() } else { NOT_DONE.into() },
                ..Default::default()
            })
            .insert(LevelId(i))
            .insert(StateLocal)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        i.to_string(),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                });
            });
    }
}

pub fn update(
    mut state: ResMut<State<GameState>>,
    levels: Res<LevelList>,
    mut level: ResMut<Level>,
    interaction_query: Query<(&LevelId, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    for (level_id, interaction) in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                *level = levels.levels[level_id.0].clone();
                state.set(GameState::ChangeLevel).unwrap();
            }
            Interaction::Hovered | Interaction::None => {}
        }
    }
}
