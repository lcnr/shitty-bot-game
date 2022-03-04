use super::buttons::*;
use super::CornerButton;
use super::ErrorText;
use super::MemUi;
use super::ERROR;
use crate::bot::BotData;
use crate::bot::BotState;
use crate::bot::ExecutionFailure;
use crate::bot::Instruction;
use crate::GameState;
use bevy::prelude::*;
use std::iter;

pub struct StopButton(Entity);
impl CornerButton for StopButton {
    const MK: fn(Entity) -> Self = StopButton;
    const MSG: &'static str = "Stop";
}

pub fn init(
    mem_ui: Res<MemUi>,
    mem: Query<&BotData>,
    children: Query<&Children>,
    mut text: Query<&mut Text>,
) {
    for mem in mem.iter() {
        let iter = iter::zip(
            &mem.instructions,
            iter::zip(&mem_ui.user_names, &mem_ui.user_values),
        );
        for (&instr, (&name, &value)) in iter {
            let text_entity = children.get(name).unwrap()[0];
            text.get_mut(text_entity).unwrap().sections[0].value =
                Instruction::from_repr(instr).map_or(String::new(), |i| i.to_string());

            let text_entity = children.get(value).unwrap()[0];
            text.get_mut(text_entity).unwrap().sections[0].value = instr.to_string();
        }
    }
}

pub fn update1(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
    stop: Res<StopButton>,
) {
    if let Ok((interaction, mut color)) = interaction_query.get_mut(stop.0) {
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

pub fn update2(
    mem_ui: Res<MemUi>,
    error: Res<ErrorText>,
    error_msg: Option<Res<ExecutionFailure>>,
    bots: Query<(&BotData, &BotState)>,
    children: Query<&Children>,
    mut text: Query<&mut Text>,
    mut color_query: Query<&mut UiColor>,
) {
    for (data, state) in bots.iter() {
        let instr = state.prev_instruction;
        let mut color = color_query
            .get_mut(mem_ui.user_names[instr as usize])
            .unwrap();
        *color = SELECTED_MEM.into();

        if Instruction::from_repr(data.instructions[state.prev_instruction as usize])
            .map_or(false, |i| i.is_wide())
        {
            let mut color = color_query
                .get_mut(mem_ui.user_values[(instr + 1 % 32) as usize])
                .unwrap();
            *color = SELECTED_MEM.into();
        }
    }

    if let Some(msg) = error_msg {
        let mut color = color_query.get_mut(error.0).unwrap();
        *color = ERROR.into();
        let text_entity = children.get(error.0).unwrap()[0];
        text.get_mut(text_entity).unwrap().sections[0].value = msg.0.clone();
    }
}
