use crate::bot::Instruction;
use std::array;
use std::num::ParseIntError;
pub struct InstructionsEditor {
    pub user_names: [String; 32],
    pub user_values: [String; 32],
    pub instructions: [u8; 32],
    active_cell: Option<(bool, usize)>,
    pub error: Option<String>,
}

impl InstructionsEditor {
    pub fn new() -> Self {
        let instructions = array::from_fn(|_| Instruction::Halt.repr());
        InstructionsEditor {
            user_names: instructions.map(|i| format!("{}", Instruction::from_repr(i).unwrap())),
            user_values: instructions.map(|i| format!("{}", i)),
            instructions,
            active_cell: None,
            error: None,
        }
    }

    pub fn active_cell_data(&mut self) -> Option<&mut String> {
        self.active_cell.map(|(b, i)| {
            if b {
                &mut self.user_names[i]
            } else {
                &mut self.user_values[i]
            }
        })
    }

    pub fn check_and_update_cell(&mut self, was_name: bool, cell: usize) {
        self.error = None;
        // normalize and parse the new value proved by the user,
        // updating the `instructions` and other part of `user_x`.
        if was_name {
            self.user_names[cell] = self.user_names[cell].trim().to_string();
            if self.user_names[cell] == "" {
                self.user_names[cell] = Instruction::from_repr(self.instructions[cell])
                    .map_or(String::new(), |i| format!("{}", i));
                return;
            }

            let mut words = self.user_names[cell].split_whitespace();
            let instr = match words.next() {
                Some("halt") => Instruction::Halt,
                Some("walk") => Instruction::Walk,
                Some("turn") => match words.next() {
                    Some("around") => Instruction::TurnAround,
                    Some("left") => Instruction::TurnLeft,
                    Some("right") => Instruction::TurnRight,
                    _ => {
                        self.error = Some(format!(
                            "invalid `turn` command, \
                            expected one of `turn around`, `turn right`, or \
                            `turn left`. found `{}`",
                            self.user_names[cell]
                        ));
                        return;
                    }
                },
                Some("wait") => Instruction::Wait,
                Some("goto") => Instruction::Goto,
                Some("if") => {
                    let mut next = words.next();
                    let negate = next == Some("not");
                    if negate {
                        next = words.next();
                    }

                    const BRANCH_COND_EXP: &str =
                        "expected one of `box`, `wall`, `edge`, or `robot`";
                    match next {
                        Some("box") => {
                            if negate {
                                Instruction::IfNotBox
                            } else {
                                Instruction::IfBox
                            }
                        }
                        Some("wall") => {
                            if negate {
                                Instruction::IfNotWall
                            } else {
                                Instruction::IfWall
                            }
                        }
                        Some("edge") => {
                            if negate {
                                Instruction::IfNotEdge
                            } else {
                                Instruction::IfEdge
                            }
                        }
                        Some("robot") => {
                            if negate {
                                Instruction::IfNotRobot
                            } else {
                                Instruction::IfRobot
                            }
                        }
                        Some(e) => {
                            self.error = Some(format!(
                                "invalid branch condition, {}, found `{}`",
                                BRANCH_COND_EXP, e
                            ));
                            return;
                        }
                        None => {
                            self.error =
                                Some(format!("missing branch condition, {}", BRANCH_COND_EXP));
                            return;
                        }
                    }
                }
                _ => {
                    self.error = Some(String::from(
                        "invalid start of command, \
                        expected one of `halt`, `walk`, `turn`, `wait`, `goto`, \
                        or `if`. For more info about the available instructions, \
                        refer to the manual.",
                    ));
                    return;
                }
            };

            if let Some(s) = words.next() {
                self.error = Some(format!(
                    "unexpected word `{}`, `{}` is already a complete instruction",
                    s, instr
                ));
                return;
            }

            self.instructions[cell] = instr.repr();
        } else {
            self.user_values[cell] = self.user_values[cell].trim().to_string();
            if self.user_values[cell] == "" {
                self.user_values[cell] = format!("{}", self.instructions[cell]);
                return;
            }
            let value = match self.user_values[cell].parse::<u8>() {
                Ok(v) => {
                    if v < 32 {
                        v
                    } else {
                        self.error = Some(format!(
                            "the value `{}` cannot be stored as it is larger than 31",
                            v
                        ));
                        return;
                    }
                }
                // TODO: improve error msg.
                Err(e) => {
                    self.error = Some(format!("{}", e));
                    return;
                }
            };

            self.instructions[cell] = value;
        }

        self.user_names[cell] = Instruction::from_repr(self.instructions[cell])
            .map_or(String::new(), |i| format!("{}", i));
        self.user_values[cell] = self.instructions[cell].to_string();
    }

    pub fn on_selection_quit(&mut self, new_cell: Option<(bool, usize)>) -> Option<(bool, usize)> {
        let old = self.active_cell.take();
        self.active_cell = new_cell;
        let (was_name, cell) = if let Some(x) = old { x } else { return None };
        self.check_and_update_cell(was_name, cell);
        old
    }
}
