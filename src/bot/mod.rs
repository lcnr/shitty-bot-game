use std::fmt::Display;

use bevy::prelude::*;

use crate::draw::{self, DrawUpdates};
use crate::Direction;
use crate::{map::*, CurrentLevel, GameState};

pub mod edit;

#[derive(Component, Copy, Clone, Debug)]
pub struct VoidedOrExited;

#[derive(Component)]
pub struct BotData {
    pub instructions: [u8; 32],
    pub start_position: GridPos,
    pub start_dir: Direction,
}

impl BotData {
    pub fn new(pos: GridPos, dir: Direction) -> Self {
        BotData {
            instructions: [0; 32],
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
    pub prev_instruction: u8,
    current_instruction: u8,
    steps: Vec<Step>,
    pub dir: Direction,
}

impl BotState {
    pub fn new(dir: Direction) -> Self {
        BotState {
            halted: false,
            prev_instruction: 0,
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

    fn read_instruction(&mut self, data: &BotData) -> Option<Instruction> {
        let instr = Instruction::from_repr(data.instructions[self.current_instruction as usize]);
        self.advance_instruction();
        instr
    }

    fn read_value(&mut self, data: &BotData) -> u8 {
        let value = data.instructions[self.current_instruction as usize];
        self.advance_instruction();
        value
    }
}

#[derive(Debug, Clone, Copy)]
#[enum_repr::EnumRepr(type = "u8", implicit = true)]
pub enum Instruction {
    Halt,
    Walk,
    TurnAround,
    TurnLeft,
    TurnRight,
    Skip,
    Goto,
    IfBox,
    IfWall,
    IfEdge,
    IfNotBox,
    IfNotWall,
    IfNotEdge,
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
                Instruction::Skip => "skip",
                Instruction::Goto => "goto",
                Instruction::IfBox => "if box",
                Instruction::IfWall => "if wall",
                Instruction::IfEdge => "if edge",
                Instruction::IfNotBox => "if not box",
                Instruction::IfNotWall => "if not wall",
                Instruction::IfNotEdge => "if not edge",
            }
        )
    }
}

impl Instruction {
    pub fn is_wide(self) -> bool {
        match self {
            Instruction::Halt
            | Instruction::Skip
            | Instruction::TurnAround
            | Instruction::TurnLeft
            | Instruction::TurnRight => false,
            Instruction::Walk
            | Instruction::Goto
            | Instruction::IfBox
            | Instruction::IfWall
            | Instruction::IfEdge
            | Instruction::IfNotBox
            | Instruction::IfNotWall
            | Instruction::IfNotEdge => true,
        }
    }

    pub fn is_positive(self) -> bool {
        match self {
            Instruction::Halt
            | Instruction::Walk
            | Instruction::TurnAround
            | Instruction::TurnLeft
            | Instruction::TurnRight
            | Instruction::Skip
            | Instruction::Goto => unreachable!(),
            Instruction::IfBox | Instruction::IfWall | Instruction::IfEdge => true,
            Instruction::IfNotBox | Instruction::IfNotWall | Instruction::IfNotEdge => false,
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
        Direction::Up => GridPos(pos.0, pos.1 - 1),
        Direction::Down => GridPos(pos.0, pos.1 + 1),
        Direction::Left => GridPos(pos.0 - 1, pos.1),
        Direction::Right => GridPos(pos.0 + 1, pos.1),
    };

    state.prev_instruction = state.current_instruction;
    let instr = if let Some(instr) = state.read_instruction(bot) {
        instr
    } else {
        state.halted = true;
        return;
    };

    match instr {
        Instruction::Halt => state.halted = true,
        Instruction::Walk => {
            let arg = state.read_value(bot);
            for _ in 0..arg {
                state.steps.push(Step::Walk);
            }
        }
        Instruction::TurnAround => {
            let new_dir = match state.dir {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };
            state.steps.push(Step::UpdateDir(new_dir));
        }
        Instruction::TurnLeft => {
            let new_dir = match state.dir {
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
            };
            state.steps.push(Step::UpdateDir(new_dir));
        }
        Instruction::TurnRight => {
            let new_dir = match state.dir {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
            };
            state.steps.push(Step::UpdateDir(new_dir));
        }
        Instruction::Skip => {
            state.steps.push(Step::Wait);
        }
        Instruction::Goto => {
            let arg = state.read_value(bot);
            state.current_instruction = arg;
        }
        Instruction::IfWall | Instruction::IfNotWall => {
            let to_jump_or_not_to_jump = instr.is_positive()
                == (matches!(map.tile(facing_grid_pos), Place::Wall)
                    || (matches!(map.tile(pos), Place::LowerFloor)
                        && matches!(map.tile(facing_grid_pos), Place::UpperFloor)));
            let target = state.read_value(bot);

            if to_jump_or_not_to_jump {
                state.current_instruction = target;
            }
        }
        Instruction::IfEdge | Instruction::IfNotEdge => {
            let to_jump_or_not_to_jump = instr.is_positive()
                == ((matches!(map.tile(pos), Place::UpperFloor)
                    && matches!(map.tile(facing_grid_pos), Place::LowerFloor))
                    || matches!(map.tile(facing_grid_pos), Place::Void));
            let target = state.read_value(bot);

            if to_jump_or_not_to_jump {
                state.current_instruction = target;
            }
        }

        Instruction::IfBox | Instruction::IfNotBox => {
            let cond =
                instr.is_positive() == matches!(entity_on_tile_facing, Some(EntityKind::Box));
            let target = state.read_value(bot);

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
        return Direction::Down;
    }
    if from.1 == to.1 + 1 {
        return Direction::Up;
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
    commands: &mut Commands,
    bot_id: Entity,
    map: &Map,
    queries: &mut QuerySet<(
        QueryState<(Entity, &BotData, &mut GridPos, &mut BotState)>,
        QueryState<(Entity, &EntityKind, &GridPos), Without<VoidedOrExited>>,
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
                blocking_entities: Query<(Entity, &EntityKind, &GridPos), Without<VoidedOrExited>>,
            ) -> Vec<(Entity, draw::Step)> {
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
                    Place::Void => matches!(tar_tile, Place::Void),
                    Place::Exit | Place::Wall => unreachable!(),
                };

                let mut steps = vec![];
                if valid_move {
                    steps.push((entity, draw::Step::Move(cur_tile_pos, tar_tile_pos)))
                }
                if let Some((e, _, pos)) = blocking_entities.iter().find(|(_, _, pos)| **pos == tar_tile_pos) && valid_move {
                    let dir = dir_to_adjacent_tile(cur_tile_pos, tar_tile_pos);
                    let new_tar_tile_pos = match dir {
                        Direction::Up => GridPos(pos.0, pos.1 - 1),
                        Direction::Down => GridPos(pos.0, pos.1 + 1),
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
                Direction::Up => GridPos(cur_grid_pos.0, cur_grid_pos.1 - 1),
                Direction::Down => GridPos(cur_grid_pos.0, cur_grid_pos.1 + 1),
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
                        commands.entity(e).insert(VoidedOrExited);
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
    q: Query<(Entity, &EntityKind, &GridPos), Without<VoidedOrExited>>,
) -> Option<EntityKind> {
    q.iter()
        .find(|(_, _, pos2)| pos == **pos2)
        .map(|(_, kind, _)| *kind)
}

pub fn progress_world(
    mut commands: Commands,
    mut render_steps: ResMut<DrawUpdates>,
    level: Res<Level>,
    mut queries: QuerySet<(
        QueryState<(Entity, &BotData, &mut GridPos, &mut BotState)>,
        QueryState<(Entity, &EntityKind, &GridPos), Without<VoidedOrExited>>,
        QueryState<&mut GridPos>,
    )>,
) {
    if let 0 = &render_steps.data.len() {
    } else {
        return;
    }

    let map = &level.map;
    let mut bots = queries
        .q0()
        .iter()
        .map(|(e, _, _, _)| e)
        .collect::<Vec<Entity>>();

    bots.sort();
    for bot_id in bots {
        let q = queries.q0();
        let (_, _, pos, state) = q.get(bot_id).unwrap();
        let viewing_pos = match state.dir {
            Direction::Up => GridPos(pos.0, pos.1 - 1),
            Direction::Down => GridPos(pos.0, pos.1 + 1),
            Direction::Left => GridPos(pos.0 - 1, pos.1),
            Direction::Right => GridPos(pos.0 + 1, pos.1),
        };
        let entity_kind = entity_on_tile(viewing_pos, queries.q1());
        let mut q = queries.q0();
        let (_, bot, pos, mut state) = q.get_mut(bot_id).unwrap();

        run_bot_interpreter(bot, *pos, &mut *state, map, entity_kind);
        let changes = apply_bot_actions(&mut commands, bot_id, map, &mut queries);
        render_steps.data.push_back(changes);
    }
}

pub fn failure_detector(
    mut commands: Commands,
    q: Query<(&GridPos, &EntityKind)>,
    bot_state: Query<&BotState>,
    level: Res<Level>,
) {
    if q.iter().any(|(pos, kind)| {
        matches!(kind, EntityKind::Robot) && matches!(level.map.tile(*pos), Place::Void)
    }) {
        commands.insert_resource(ExecutionFailure(format!(
            "stage failed: the robot fell into the void and will not make further progress"
        )));
    } else if q.iter().any(|(pos, kind)| {
        matches!(kind, EntityKind::Robot) && matches!(level.map.tile(*pos), Place::Exit)
    }) {
        commands.insert_resource(ExecutionFailure(format!(
            "stage failed: the robot entered the exit without first inserting all boxes"
        )));
    } else if bot_state.iter().any(|st| st.halted) {
        commands.insert_resource(ExecutionFailure(format!(
            "stage failed: the robot halted and will not make further progress"
        )));
    } else if q.iter().any(|(pos, kind)| {
        matches!(kind, EntityKind::Box) && matches!(level.map.tile(*pos), Place::Void)
    }) {
        commands.insert_resource(ExecutionFailure(format!(
            "stage failed: a box fell into the void prevent a successful finish"
        )));
    }
}

pub fn level_complete_checker(
    mut state: ResMut<State<GameState>>,
    q: Query<(&GridPos, &EntityKind)>,
    level: Res<Level>,
    mut level_list: ResMut<LevelList>,
    current_level: Res<CurrentLevel>,
) {
    let level_won = q
        .iter()
        .all(|(pos, _)| matches!(level.map.tile(*pos), Place::Exit));
    if level_won {
        level_list.beaten[current_level.0] = true;
        state.set(GameState::StartScreen).unwrap();
    }
}

pub fn init_state(mut commands: Commands, bots: Query<(Entity, &BotData)>) {
    for (e, data) in bots.iter() {
        commands.entity(e).insert(BotState::new(data.start_dir));
    }
}

pub struct ExecutionFailure(pub String);
