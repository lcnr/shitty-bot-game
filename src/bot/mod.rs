use std::fmt::Display;

use bevy::prelude::*;

use crate::draw::{self, DrawUpdates};
use crate::map::*;
use crate::Direction;

pub mod edit;

#[derive(Component)]
pub struct BotData {
    pub instructions: [u8; 32],
    pub start_position: GridPos,
    pub start_dir: Direction,
}

pub enum Memory {
    Data(u8),
    Instruction(Instruction),
}

impl BotData {
    pub fn from_iter<I: IntoIterator<Item = Memory>>(
        pos: GridPos,
        dir: Direction,
        iter: I,
    ) -> Self {
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
        BotData {
            instructions: instructions.try_into().expect("too long"),
            start_position: pos,
            start_dir: dir,
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
    pub dir: Direction,
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

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Instruction::Halt => "halt",
                Instruction::Walk => "walk",
                Instruction::TurnAround => "turn around",
                Instruction::TurnLeft => "turn left",
                Instruction::TurnRight => "turn right",
                Instruction::Wait => "wait",
                Instruction::Goto => "goto",
                Instruction::IfBox => "if box",
                Instruction::IfWall => "if wall",
                Instruction::IfEdge => "if edge",
                Instruction::IfRobot => "if robot",
                Instruction::IfNotBox => "if not box",
                Instruction::IfNotWall => "if not wall",
                Instruction::IfNotEdge => "if not edge",
                Instruction::IfNotRobot => "if not robot",
            }
        )
    }
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

pub fn run_bot_interpreter(
    bot: &BotData,
    pos: GridPos,
    state: &mut BotState,
    map: &Map,
    entity_on_tile_facing: Option<EntityKind>,
) {
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
            let to_jump_or_not_to_jump =
                instr.is_positive() == matches!(map.tile(facing_grid_pos), Place::Wall);
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

        Instruction::IfBox | Instruction::IfNotBox => {
            let cond =
                instr.is_positive() == matches!(entity_on_tile_facing, Some(EntityKind::Box));
            let target = {
                state.advance_instruction();
                bot.instructions[state.current_instruction as usize]
            };

            if cond {
                state.current_instruction = target;
            }
        }
        Instruction::IfRobot | Instruction::IfNotRobot => {
            let cond =
                instr.is_positive() == matches!(entity_on_tile_facing, Some(EntityKind::Robot));
            let target = {
                state.advance_instruction();
                bot.instructions[state.current_instruction as usize]
            };

            if cond {
                state.current_instruction = target;
            }
        }
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
        QueryState<(Entity, &BotData, &mut GridPos, &mut BotState)>,
        QueryState<(Entity, &EntityKind, &GridPos)>,
        QueryState<&mut GridPos>,
    )>,
) -> Vec<(Entity, draw::Step)> {
    let mut render_steps = vec![];

    let mut q = queries.q0();
    let (_, _, cur_grid_pos, mut state) = q.get_mut(bot_id).unwrap();
    let bot_action = if let Some(action) = state.steps.pop() {
        action
    } else {
        return Vec::new();
    };

    match bot_action {
        Step::Wait => render_steps.push((bot_id, draw::Step::Idle)),
        Step::Walk => {
            fn is_valid_move(
                entity: Entity,
                cur_tile_pos: GridPos,
                cur_tile: Place,
                tar_tile_pos: GridPos,
                tar_tile: Place,
                map: &Map,
                blocking_entities: Query<(Entity, &EntityKind, &GridPos)>,
            ) -> Vec<(Entity, draw::Step)> {
                eprintln!(
                    "is_valid_move({:?}, {:?}, {:?}, {:?})",
                    cur_tile_pos, cur_tile, tar_tile_pos, tar_tile
                );
                let valid_move = match cur_tile {
                    Place::UpperFloor => match tar_tile {
                        Place::LowerFloor | Place::UpperFloor | Place::Void | Place::Exit => true,
                        Place::Ramp(ramp_dir) => {
                            dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos) == ramp_dir
                        }
                        Place::Wall => false,
                    },
                    Place::LowerFloor => match tar_tile {
                        Place::Void | Place::LowerFloor | Place::Exit => true,
                        Place::Ramp(ramp_dir) => {
                            dir_to_adjacent_tile(tar_tile_pos, cur_tile_pos) == ramp_dir
                        }
                        Place::UpperFloor | Place::Wall => false,
                    },
                    Place::Ramp(ramp_dir) => match tar_tile {
                        Place::Void | Place::Exit => true,
                        Place::LowerFloor => {
                            dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos) == ramp_dir
                        }
                        Place::UpperFloor => {
                            dir_to_adjacent_tile(tar_tile_pos, cur_tile_pos) == ramp_dir
                        }
                        Place::Ramp(tar_ramp_dir) => {
                            is_dirs_opposite(ramp_dir, tar_ramp_dir)
                                && (dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos) == ramp_dir
                                    || dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos) == ramp_dir)
                        }
                        Place::Wall => false,
                    },
                    Place::Void | Place::Exit | Place::Wall => unreachable!(),
                };

                let mut steps = vec![];
                if valid_move {
                    steps.push((entity, draw::Step::Move(cur_tile_pos, tar_tile_pos)))
                }
                if let Some((e, _, pos)) = blocking_entities.iter().find(|(_, _, pos)| **pos == tar_tile_pos) && valid_move {
                    let dir = dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos);
                    let new_tar_tile_pos = match dir {
                        Direction::Up => GridPos(pos.0, pos.1 + 1),
                        Direction::Down => GridPos(pos.0, pos.1 - 1),
                        Direction::Left => GridPos(pos.0 - 1, pos.1),
                        Direction::Right => GridPos(pos.0 + 1, pos.1),
                    };
                    let new_tar_tile = map.tile(new_tar_tile_pos);
                    match &*is_valid_move(e, tar_tile_pos, tar_tile, new_tar_tile_pos, new_tar_tile, map, blocking_entities) {
                        [] => steps.clear(),
                        nested_steps @ [..] => steps.extend(nested_steps),
                    };
                }
                steps
            }

            let tar_grid_pos = match state.dir {
                Direction::Up => GridPos(cur_grid_pos.0, cur_grid_pos.1 + 1),
                Direction::Down => GridPos(cur_grid_pos.0, cur_grid_pos.1 - 1),
                Direction::Left => GridPos(cur_grid_pos.0 - 1, cur_grid_pos.1),
                Direction::Right => GridPos(cur_grid_pos.0 + 1, cur_grid_pos.1),
            };
            let cur_tile = map.tile(*cur_grid_pos);
            let tar_tile = map.tile(tar_grid_pos);
            let steps = is_valid_move(
                bot_id,
                *cur_grid_pos,
                cur_tile,
                tar_grid_pos,
                tar_tile,
                map,
                queries.q1(),
            );
            dbg!(&steps);

            if let [] = &*steps {
                render_steps.push((bot_id, draw::Step::MoveFail))
            }

            for (e, step) in steps {
                render_steps.push((e, step));
                if let draw::Step::Move(_, tar_pos) = step {
                    if let Place::Void | Place::Exit = map.tile(tar_pos) {
                        let mut q = queries.q0();
                        if let Ok((_, _, _, mut state)) = q.get_mut(e) {
                            state.steps.clear();
                            state.halted = true;
                        }
                    }
                    let mut q = queries.q2();
                    *q.get_mut(e).unwrap() = tar_pos;
                }
            }
        }
        Step::UpdateDir(dir) => {
            render_steps.push((bot_id, draw::Step::UpdateDir(state.dir, dir)));
            state.dir = dir;
        }
    }

    render_steps
}

pub fn entity_on_tile(
    pos: GridPos,
    q: Query<(Entity, &EntityKind, &GridPos)>,
) -> Option<EntityKind> {
    q.iter()
        .find(|(_, _, pos2)| pos == **pos2)
        .map(|(_, kind, _)| *kind)
}

pub fn progress_world(
    mut render_steps: ResMut<DrawUpdates>,
    map: Res<Map>,
    mut queries: QuerySet<(
        QueryState<(Entity, &BotData, &mut GridPos, &mut BotState)>,
        QueryState<(Entity, &EntityKind, &GridPos)>,
        QueryState<&mut GridPos>,
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
        let (_, _, pos, _) = q.get(bot_id).unwrap();
        let entity_kind = entity_on_tile(*pos, queries.q1());
        let mut q = queries.q0();
        let (_, bot, pos, mut state) = q.get_mut(bot_id).unwrap();

        run_bot_interpreter(bot, *pos, &mut *state, map, entity_kind);
        let changes = apply_bot_actions(bot_id, map, &mut queries);
        render_steps.data.push_back(changes);
    }
}
