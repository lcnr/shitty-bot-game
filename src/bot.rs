use bevy::prelude::*;

use crate::draw::{self, DrawUpdates};
use crate::map::*;
use crate::Direction;

#[derive(Component)]
pub struct Bot {
    instructions: [u8; 32],
}

pub enum Memory {
    Data(u8),
    Instruction(Instruction),
}

impl Bot {
    pub fn from_iter<I: IntoIterator<Item = Memory>>(iter: I) -> Self {
        let mut instructions = Vec::new();
        for mem in iter {
            match mem {
                Memory::Data(val) => {
                    assert!(val < 32);
                    instructions.push(val);
                }
                Memory::Instruction(instr) => {
                    instructions.push(instr.repr());
                }
            }
        }

        while instructions.len() < 32 {
            instructions.push(Instruction::Halt.repr());
        }
        Bot {
            instructions: instructions.try_into().expect("too long"),
        }
    }
}

#[derive(Debug)]
pub enum Step {
    Wait,
    Walk,
    UpdateDir(Direction),
}

#[derive(Debug, Component)]
pub struct BotState {
    halted: bool,
    current_instruction: u8,
    steps: Vec<Step>,
    dir: Direction,
}

impl BotState {
    pub fn new(dir: Direction) -> Self {
        BotState {
            halted: false,
            current_instruction: 0,
            steps: Vec::new(),
            dir,
        }
    }

    fn advance_instruction(&mut self) {
        if self.current_instruction == 31 {
            self.current_instruction = 0;
        } else {
            self.current_instruction += 1;
        }
    }
}

#[derive(Clone, Copy)]
#[enum_repr::EnumRepr(type = "u8", implicit = true)]
pub enum Instruction {
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

impl Instruction {
    pub fn is_positive(self) -> bool {
        match self {
            Instruction::Halt
            | Instruction::Walk
            | Instruction::TurnAround
            | Instruction::TurnLeft
            | Instruction::TurnRight
            | Instruction::Wait
            | Instruction::Goto => unreachable!(),
            Instruction::IfBox
            | Instruction::IfWall
            | Instruction::IfEdge
            | Instruction::IfRobot => true,
            Instruction::IfNotBox
            | Instruction::IfNotWall
            | Instruction::IfNotEdge
            | Instruction::IfNotRobot => false,
        }
    }
}

pub fn run_bot_interpreter(bot: &Bot, pos: GridPos, state: &mut BotState, map: &Map) {
    if state.halted || state.steps.len() != 0 {
        return;
    }

    let facing_grid_pos = match state.dir {
        Direction::Up => GridPos(pos.0, pos.1 + 1),
        Direction::Down => GridPos(pos.0, pos.1 - 1),
        Direction::Left => GridPos(pos.0 - 1, pos.1),
        Direction::Right => GridPos(pos.0 + 1, pos.1),
    };

    let instr =
        Instruction::from_repr(bot.instructions[state.current_instruction as usize]).unwrap();
    match instr {
        Instruction::Halt => state.halted = true,
        Instruction::Walk => {
            let arg = {
                state.advance_instruction();
                bot.instructions[state.current_instruction as usize]
            };
            for _ in 0..arg {
                state.steps.push(Step::Walk);
            }
            println!("{:?}", state.steps);
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
        Instruction::IfWall | Instruction::IfNotWall => {
            let to_jump_or_not_to_jump = instr.is_positive() == matches!(map.tile(facing_grid_pos), Place::Wall);
            let target = {
                state.advance_instruction();
                bot.instructions[state.current_instruction as usize]
            };

            if to_jump_or_not_to_jump {
                state.current_instruction = target;
            }
        }
        Instruction::IfEdge | Instruction::IfNotEdge => {
            let to_jump_or_not_to_jump = instr.is_positive()
                == (matches!(map.tile(pos), Place::UpperFloor)
                    && matches!(map.tile(facing_grid_pos), Place::LowerFloor))
                || matches!(map.tile(pos), Place::Void);
            let target = {
                state.advance_instruction();
                bot.instructions[state.current_instruction as usize]
            };

            if to_jump_or_not_to_jump {
                state.current_instruction = target;
            }
        }

        Instruction::IfBox => todo!(),
        Instruction::IfNotBox => todo!(),
        Instruction::IfRobot => todo!(),
        Instruction::IfNotRobot => todo!(),
    }
}

fn dir_to_adjacent_tile(from: GridPos, to: GridPos) -> Direction {
    if from.0 + 1 == to.0 {
        return Direction::Right;
    }
    if from.0 == to.0 + 1 {
        return Direction::Left;
    }
    if from.1 + 1 == to.1 {
        return Direction::Up;
    }
    if from.1 == to.1 + 1 {
        return Direction::Down;
    }
    panic!("bad inputs {:?} {:?}", from, to);
}

fn is_dirs_opposite(d1: Direction, d2: Direction) -> bool {
    match (d1, d2) {
        (Direction::Up, Direction::Down)
        | (Direction::Down, Direction::Up)
        | (Direction::Left, Direction::Right)
        | (Direction::Right, Direction::Left) => true,
        _ => false,
    }
}

fn apply_bot_actions(
    bot_id: Entity,
    map: &Map,
    queries: &mut QuerySet<(
        QueryState<(Entity, &Bot, &mut GridPos, &mut BotState)>,
        QueryState<(&EntityKind, &GridPos)>,
    )>,
) -> Vec<(Entity, draw::Step)> {
    let mut render_steps = vec![];

    let mut q = queries.q0();
    let (_, _, mut cur_grid_pos, mut state) = q.get_mut(bot_id).unwrap();
    let bot_action = if let Some(action) = state.steps.pop() {
        action
    } else {
        return Vec::new()
    };
    

    match bot_action {
        Step::Wait => (),
        Step::Walk => {
            let tar_grid_pos = match state.dir {
                Direction::Up => GridPos(cur_grid_pos.0, cur_grid_pos.1 + 1),
                Direction::Down => GridPos(cur_grid_pos.0, cur_grid_pos.1 - 1),
                Direction::Left => GridPos(cur_grid_pos.0 - 1, cur_grid_pos.1),
                Direction::Right => GridPos(cur_grid_pos.0 + 1, cur_grid_pos.1),
            };

            let cur_tile = map.tile(*cur_grid_pos);
            let tar_tile = map.tile(tar_grid_pos);
            println!("{:?} {:?}", cur_tile, tar_tile);

            let valid_move = match cur_tile {
                Place::UpperFloor => match tar_tile {
                    Place::LowerFloor | Place::UpperFloor | Place::Void | Place::Exit => true,
                    Place::Ramp(ramp_dir) => {
                        dir_to_adjacent_tile(*cur_grid_pos, tar_grid_pos) == ramp_dir
                    }
                    Place::Wall => false,
                },
                Place::LowerFloor => match tar_tile {
                    Place::Void | Place::LowerFloor | Place::Exit => true,
                    Place::Ramp(ramp_dir) => {
                        dir_to_adjacent_tile(tar_grid_pos, *cur_grid_pos) == ramp_dir
                    }
                    Place::UpperFloor | Place::Wall => false,
                },
                Place::Ramp(ramp_dir) => match tar_tile {
                    Place::Void | Place::Exit => true,
                    Place::LowerFloor => {
                        dir_to_adjacent_tile(*cur_grid_pos, tar_grid_pos) == ramp_dir
                    }
                    Place::UpperFloor => {
                        dir_to_adjacent_tile(tar_grid_pos, *cur_grid_pos) == ramp_dir
                    }
                    Place::Ramp(tar_ramp_dir) => {
                        is_dirs_opposite(ramp_dir, tar_ramp_dir)
                            && (dir_to_adjacent_tile(*cur_grid_pos, tar_grid_pos) == ramp_dir
                                || dir_to_adjacent_tile(*cur_grid_pos, tar_grid_pos) == ramp_dir)
                    }
                    Place::Wall => false,
                },
                Place::Void | Place::Wall | Place::Exit => unreachable!(),
            };
            
            /*
                this is where logic should go of checking if `tar_tile` is occupied by
                a bot or a box. if it is we should effectively function-ify this match arm
                and recurse, attempting to move the bot/box in the same direction as we are facing
            */

            dbg!(valid_move);
            if valid_move {
                render_steps.push((bot_id, draw::Step::Move(*cur_grid_pos, tar_grid_pos)));
                *cur_grid_pos = tar_grid_pos;
            } else {
                render_steps.push((bot_id, draw::Step::MoveFail));
            }

        }
        Step::UpdateDir(dir) => {
            render_steps.push((bot_id, draw::Step::UpdateDir(state.dir, dir)));
            state.dir = dir;
        }
    }

    render_steps
}

pub fn progress_world(
    mut render_steps: ResMut<DrawUpdates>,
    map: Res<Map>,
    mut queries: QuerySet<(
        QueryState<(Entity, &Bot, &mut GridPos, &mut BotState)>,
        QueryState<(&EntityKind, &GridPos)>,
    )>,
) {
    if let 0 = &render_steps.data.len() {
    } else {
        return;
    }

    // `Res<T>: Copy` cannot be proven ???
    let map = &*map;
    let mut bots = queries
        .q0()
        .iter()
        .map(|(e, _, _, _)| e)
        .collect::<Vec<Entity>>();

    bots.sort();
    for bot_id in bots {
        let mut q = queries.q0();
        let (_, bot, pos, mut state) = q.get_mut(bot_id).unwrap();
        run_bot_interpreter(bot, *pos, &mut *state, map);
        let changes = apply_bot_actions(bot_id, map, &mut queries);
        render_steps.data.push_back(changes);
    }
}
