use super::buttons::*;
use super::CornerButton;
use crate::util::StateLocal;
use crate::GameState;
use bevy::prelude::*;

pub struct StopButton(Entity);
impl CornerButton for StopButton {
    const MK: fn(Entity) -> Self = StopButton;
    const MSG: &'static str = "Stop";
}

pub fn update(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
    stop: Res<StopButton>,
) {
    if let Ok((interaction, mut color, children)) = interaction_query.get_mut(stop.0) {
        match *interaction {
            Interaction::Clicked => {
                state.set(GameState::Programming).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
