use bevy::prelude::*;

use crate::map::*;
use crate::Direction;

#[derive(Component)]
pub struct Bot {
    instructions: [u8; 32],
}

enum Step {
    Wait,
    Walk,
    UpdateDir(Direction),
}

#[derive(Component)]
pub struct BotState {
    halted: bool,
    current_instruction: u8,
    steps: Vec<Step>,
    dir: Direction,
}

impl BotState {
    fn advance_instruction(&mut self) {
        if self.current_instruction == 31 {
            self.current_instruction = 0;
        } else {
            self.current_instruction += 1;
        }
    }
}

#[enum_repr::EnumRepr(type = "u8", implicit = true)]
enum Instruction {
    Skip,
    Halt,
    Walk,
    TurnAround,
    TurnLeft,
    TurnRight,
    Wait,
    Goto,
    IfBox,
    IfWall,
    IfEdge,
    IfRobot,
    IfNotBox,
    IfNotWall,
    IfNotEdge,
    IfNotRobot,
}

pub fn run_bot_interpreter(bot: Entity, map: &Map, mut bots: Query<(&Bot, &mut BotState)>) {
    if let Some((bot, mut state)) = bots.get_mut(bot).ok().and_then(|(bot, state)| {
        match state.steps.len() == 0 && state.halted == false {
            true => Some((bot, state)),
            false => None,
        }
    }) {
        match Instruction::from_repr(bot.instructions[state.current_instruction as usize]).unwrap()
        {
            Instruction::Skip => state.advance_instruction(),
            Instruction::Halt => state.halted = true,
            Instruction::Walk => {
                let arg = {
                    state.advance_instruction();
                    bot.instructions[state.current_instruction as usize]
                };
                for _ in 0..arg {
                    state.steps.push(Step::Walk);
                }
                state.advance_instruction();
            }
            Instruction::TurnAround => {
                let new_dir = match state.dir {
                    Direction::Up => Direction::Down,
                    Direction::Down => Direction::Up,
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Right,
                };
                state.steps.push(Step::UpdateDir(new_dir));
                state.advance_instruction();
            }
            Instruction::TurnLeft => {
                let new_dir = match state.dir {
                    Direction::Up => Direction::Left,
                    Direction::Down => Direction::Right,
                    Direction::Left => Direction::Down,
                    Direction::Right => Direction::Up,
                };
                state.steps.push(Step::UpdateDir(new_dir));
                state.advance_instruction();
            }
            Instruction::TurnRight => {
                let new_dir = match state.dir {
                    Direction::Up => Direction::Right,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Up,
                    Direction::Right => Direction::Down,
                };
                state.steps.push(Step::UpdateDir(new_dir));
                state.advance_instruction();
            }
            Instruction::Wait => {
                let arg = {
                    state.advance_instruction();
                    bot.instructions[state.current_instruction as usize]
                };
                for _ in 0..arg {
                    state.steps.push(Step::Wait);
                }
                state.advance_instruction();
            }
            Instruction::Goto => {
                let arg = {
                    state.advance_instruction();
                    bot.instructions[state.current_instruction as usize]
                };
                state.current_instruction = arg;
            }
            Instruction::IfBox => todo!(),
            Instruction::IfWall => todo!(),
            Instruction::IfEdge => todo!(),
            Instruction::IfRobot => todo!(),
            Instruction::IfNotBox => todo!(),
            Instruction::IfNotWall => todo!(),
            Instruction::IfNotEdge => todo!(),
            Instruction::IfNotRobot => todo!(),
        }
    }
}

fn apply_bot_actions(
    bot: Entity,
    map: &Map,
    queries: &mut QuerySet<(
        QueryState<(&Bot, &mut GridPos, &mut BotState)>,
        QueryState<(Entity, &mut BotState)>,
        QueryState<(Option<&BotState>, &GridPos)>,
    )>,
) -> Vec<Step> {
    let mut render_steps = vec![];

    let (bot, grid_pos, state) = queries.q1().get_mut(bot).unwrap();
    let bot_action = state.steps.pop().unwrap();

    match bot_action {
        Step::Wait => (),
        Step::Walk => {
            render_steps.push(Step::Walk);
            let target_grid_pos = match state.dir {
                Direction::Up => GridPos(grid_pos.0, grid_pos.1 + 1),
                Direction::Down => GridPos(grid_pos.0, grid_pos.1 - 1),
                Direction::left => GridPos(grid_pos.0 - 1, grid_pos.1),
                Direction::Right => GridPos(grid_pos.0 + 1, grid_pos.1),
            };

            if target_grid_pos.0 < 0 
                || target_grid_pos.0 >= map.width
                || target_grid_pos.1 < 0
                || target_grid_pos.1 >= map.height {
                    *grid_pos = target_grid_pos;
                } 
        }
        Step::UpdateDir(dir) => {
            render_steps.push(Step::UpdateDir(dir));
            state.dir = dir;
        },
    }

    render_steps
}

pub fn progress_world(
    map: Res<Map>,
    mut queries: QuerySet<(
        QueryState<(&Bot, &mut BotState)>,
        QueryState<(Entity, &mut BotState)>,
        QueryState<(Option<&BotState>, &GridPos)>,
    )>,
) {
    // `Res<T>: Copy` cannot be proven ??? 
    let map = &*map;
    let mut bots = queries
        .q1()
        .iter()
        .filter(|(_, _, state)| state.halted == false)
        .map(|(e, _)| e)
        .collect::<Vec<Entity>>();

    bots.sort();
    for bot in bots {
        run_bot_interpreter(bot, map, queries.q0());
        let changes = apply_bot_actions(bot, map, &mut queries);
    }
}
