use bevy::prelude::*;

use crate::draw::DrawUpdates;
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

pub fn run_bot_interpreter(bot: &Bot, state: &mut BotState, map: &Map) {
    match Instruction::from_repr(bot.instructions[state.current_instruction as usize]).unwrap() {
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

fn apply_bot_actions(
    bot: Entity,
    map: &Map,
    queries: &mut QuerySet<(
        QueryState<(Entity, &Bot, &mut GridPos, &mut BotState)>,
        QueryState<(Option<&BotState>, &GridPos)>,
    )>,
) -> Vec<Step> {
    let mut render_steps = vec![];

    let (_, bot, grid_pos, state) = queries.q0().get_mut(bot).unwrap();
    let bot_action = state.steps.pop().unwrap();

    match bot_action {
        Step::Wait => (),
        Step::Walk => {
            render_steps.push(Step::Walk);
            let target_grid_pos = match state.dir {
                Direction::Up => GridPos(grid_pos.0, grid_pos.1 + 1),
                Direction::Down => GridPos(grid_pos.0, grid_pos.1 - 1),
                Direction::Left => GridPos(grid_pos.0 - 1, grid_pos.1),
                Direction::Right => GridPos(grid_pos.0 + 1, grid_pos.1),
            };

            let cur_tile = map.tile(grid_pos.0, grid_pos.1);
            let tar_tile = map.tile(target_grid_pos.0, target_grid_pos.1);

            let valid_move = match cur_tile {
                Place::UpperFloor => match tar_tile {
                    // allowed
                    Place::UpperFloor => (),
                    Place::Ramp(Direction) => (),
                    Place::Void => true,
                    Place::LowerFloor => (),
                    Place::Exit => (),

                    // disallowed
                    Place::Wall => false,
                },
                Place::LowerFloor => match tar_tile {
                    // allowed
                    Place::Ramp(Direction) => (),
                    Place::Void => true,
                    Place::LowerFloor => (),
                    Place::Exit => (),
                    // disallowed
                    Place::UpperFloor | Place::Wall => false,
                },
                Place::Ramp(Direction) => match tar_tile {
                    // allowed
                    Place::Ramp(Direction) => (),
                    Place::Void => (),
                    Place::LowerFloor => (),
                    Place::UpperFloor => (),
                    Place::Exit => (),
                    Place::Ramp(Direction) => (),
                    // disallowed
                    Place::Wall => false,
                },
                Place::Void | Place::Wall | Place::Exit => unreachable!(),
            };

            if target_grid_pos.0 < 0
                || target_grid_pos.0 >= map.width
                || target_grid_pos.1 < 0
                || target_grid_pos.1 >= map.height
            {
                *grid_pos = target_grid_pos;
            }
        }
        Step::UpdateDir(dir) => {
            render_steps.push(Step::UpdateDir(dir));
            state.dir = dir;
        }
    }

    render_steps
}

pub fn progress_world(
    render_steps: ResMut<DrawUpdates>,
    map: Res<Map>,
    mut queries: QuerySet<(
        QueryState<(Entity, &Bot, &mut GridPos, &mut BotState)>,
        QueryState<(Option<&BotState>, &GridPos)>,
    )>,
) {
    // `Res<T>: Copy` cannot be proven ???
    let map = &*map;
    let mut bots = queries
        .q0()
        .iter()
        .filter(|(_, _, _, state)| state.halted == false)
        .map(|(e, _, _, _)| e)
        .collect::<Vec<Entity>>();

    bots.sort();
    for bot_id in bots {
        let (_, bot, _, state) = queries.q0().get_mut(bot_id).unwrap();
        run_bot_interpreter(bot, &mut *state, map);
        let changes = apply_bot_actions(bot_id, map, &mut queries);
        // render_steps.push(changes);
    }
}
