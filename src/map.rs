use crate::Direction;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Place {
    UpperFloor,
    LowerFloor,
    Ramp(Direction),
    Void,
    Wall,
    Exit,
}

#[derive(Clone, Debug)]
pub struct LevelList {
    pub levels: Vec<Level>,
    pub beaten: Vec<bool>,
}

pub fn read_levels(file: &str) -> LevelList {
    let string = std::fs::read_to_string(file).unwrap();
    let level_list: Vec<LevelSerde> = serde_json::from_str(&string).unwrap();

    let beaten = vec![false; level_list.len()];
    LevelList {
        levels: level_list
            .into_iter()
            .map(Level::from_level_serde)
            .collect(),
        beaten,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelSerde {
    pub map: String,
    pub boxes: Vec<(usize, usize)>,
    pub bots: Vec<(usize, usize, Direction)>,
}

#[derive(Clone, Debug)]
pub struct Level {
    pub map: Map,
    pub boxes: Vec<GridPos>,
    pub bots: Vec<(GridPos, Direction)>,
}

impl Level {
    fn from_level_serde(level_serde: LevelSerde) -> Self {
        let map_str =
            std::fs::read_to_string(format!("assets/levels/{}.map", level_serde.map)).unwrap();
        let map = Map::from_str(&map_str);
        Level {
            map,
            boxes: level_serde
                .boxes
                .into_iter()
                .map(|(x, y)| GridPos(x, y))
                .collect(),
            bots: level_serde
                .bots
                .into_iter()
                .map(|(x, y, dir)| (GridPos(x, y), dir))
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub layout: Vec<Place>,
}

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Map {}x{}:", self.width, self.height))?;
        for chunk in self.layout.chunks(self.width) {
            let mut f = f.debug_list();
            f.entries(chunk.iter()).finish()?;
        }
        Ok(())
    }
}

impl Map {
    pub fn from_str(src: &str) -> Self {
        let mut lines = Vec::new();
        for l in src.lines() {
            let l = l.trim_end();
            let mut line = Vec::new();
            for c in l.chars() {
                line.push(match c {
                    '-' => Place::UpperFloor,
                    '.' => Place::LowerFloor,
                    '^' => Place::Ramp(Direction::Up),
                    'v' => Place::Ramp(Direction::Down),
                    '<' => Place::Ramp(Direction::Left),
                    '>' => Place::Ramp(Direction::Right),
                    ' ' => Place::Void,
                    '#' => Place::Wall,
                    'o' => Place::Exit,
                    _ => panic!("unexpected {:?}", c),
                });
            }
            lines.push(line);
        }

        let width = lines.iter().map(|l| l.len()).max().unwrap();
        let mut layout = Vec::with_capacity(width * lines.len());
        for line in lines.iter() {
            let len = line.len();
            layout.extend(line.iter().copied());
            for _ in len..width {
                layout.push(Place::Void);
            }
        }

        Map {
            width,
            height: lines.len(),
            layout,
        }
    }

    pub fn tile(&self, GridPos(x, y): GridPos) -> Place {
        if x < self.width && y < self.height {
            self.layout[y * self.width + x]
        } else {
            Place::Void
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]

pub enum EntityKind {
    Robot,
    Box,
}

#[derive(Copy, Clone, Component)]
pub struct BoxData {
    pub start_position: GridPos,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct GridPos(pub usize, pub usize);
