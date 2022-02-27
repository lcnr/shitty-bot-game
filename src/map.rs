use crate::Direction;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Place {
    UpperFloor,
    LowerFloor,
    Ramp(Direction),
    Void,
    Wall,
    Exit,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub layout: Vec<Place>,
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
                    'X' => Place::Wall,
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

    pub fn dummy_new() -> Self {
        Map::from_str(include_str!("../example.map"))
    }

    pub fn tile(&self, GridPos(x, y): GridPos) -> Place {
        assert!(x < self.width && y < self.height);
        self.layout[y * self.width + x]
    }
}

#[derive(Component)]

pub enum EntityKind {
    Robot,
    Box,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct GridPos(pub usize, pub usize);
