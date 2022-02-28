use std::collections::VecDeque;
use std::ops::Add;
use std::ops::Deref;
use std::ops::Mul;

use crate::bot::BotData;
use crate::bot::BotState;
use crate::map::EntityKind;
use crate::map::GridPos;
use crate::map::Map;
use crate::map::Place;
use crate::Direction;
use bevy::prelude::*;

mod mesh;

const UPPER_FLOOR: f32 = 0.6;
const LOWER_FLOOR: f32 = 0.1;

pub struct DrawUpdates {
    pub data: VecDeque<Vec<(Entity, Step)>>,
}

#[derive(Clone, Copy, Debug)]
pub enum Step {
    Idle,
    Move(GridPos, GridPos),
    MoveFail,
    UpdateDir(Direction, Direction),
}

impl DrawUpdates {
    pub fn empty() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }
}

pub fn init_map_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
    entities: Query<(Entity, &EntityKind, &GridPos)>,
    robots: Query<&BotData>,
) {
    commands.insert_resource(DrawTimer(Timer::from_seconds(0.0, false)));

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_xyz(0.7, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(map.width as f32, map.height as f32, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });

    let box_xy = shape::Box {
        min_x: -0.5,
        max_x: 0.5,
        min_y: -0.5,
        max_y: 0.5,
        min_z: 0.0,
        max_z: 1.0,
    };

    for x in 0..map.width {
        for y in 0..map.height {
            let transform = Transform::from_xyz(
                x as f32 - map.width as f32 / 2.0,
                y as f32 - map.height as f32 / 2.0,
                0.0,
            );
            match map.tile(GridPos(x, y)) {
                Place::UpperFloor => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: UPPER_FLOOR,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::LowerFloor => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: LOWER_FLOOR,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::Ramp(dir) => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh::slope_mesh(dir)),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::Void => {}
                Place::Wall => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: 0.9,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.7, 0.9, 0.7).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::Exit => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: 1.0,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.1, 0.9, 0.1).into()),
                        transform,
                        ..Default::default()
                    });
                }
            }
        }
    }

    for (entity, kind, &position) in entities.iter() {
        let transform = Transform::from_translation(pos_to_world(&map, position));

        match kind {
            EntityKind::Robot => {
                let state = robots.get(entity).expect("bot without bot state");
                commands.get_or_spawn(entity).insert_bundle(PbrBundle {
                    mesh: meshes.add(mesh::robot_mesh()),
                    material: materials.add(Color::rgb(0.25, 0.12, 0.1).into()),
                    transform: transform
                        .with_rotation(Quat::from_rotation_z(dir_to_radians(state.start_dir))),
                    ..Default::default()
                });
            }
            EntityKind::Box => {
                commands.get_or_spawn(entity).insert_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box {
                        min_x: -0.4,
                        max_x: 0.4,
                        min_y: -0.4,
                        max_y: 0.4,
                        min_z: 0.0,
                        max_z: 0.8,
                    })),
                    material: materials.add(Color::rgb(0.25, 0.12, 0.1).into()),
                    transform,
                    ..Default::default()
                });
            }
        }
    }
}

pub struct DrawTimer(Timer);
impl Deref for DrawTimer {
    type Target = Timer;
    fn deref(&self) -> &Timer {
        &self.0
    }
}

pub fn update_map_system(
    mut events: ResMut<DrawUpdates>,
    time: Res<Time>,
    mut timer: ResMut<DrawTimer>,
    map: Res<Map>,
    mut transforms: Query<&mut Transform>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        events.data.pop_front();
        timer.0 = Timer::from_seconds(0.5, false);
    }

    let steps = if let Some(steps) = events.data.front() {
        steps
    } else {
        return;
    };

    for &(entity, step) in steps {
        match step {
            Step::Idle => {}
            Step::Move(from, to) => {
                let mut transform = transforms.get_mut(entity).expect("sus step");
                let old_pos = pos_to_world(&map, from);
                let new_pos = pos_to_world(&map, to);
                let position = interpolate(timer.percent(), old_pos, new_pos);
                *transform = transform.with_translation(position);
            }
            Step::MoveFail => {}
            Step::UpdateDir(old, new) => {
                let mut transform = transforms.get_mut(entity).expect("sus step");
                let mut old_rad = dir_to_radians(old);
                let new_rad = dir_to_radians(new);
                if new_rad - old_rad > std::f32::consts::PI {
                    old_rad += std::f32::consts::PI * 2.0;
                } else if old_rad - new_rad > std::f32::consts::PI {
                    old_rad -= std::f32::consts::PI * 2.0;
                }
                let angle = interpolate(timer.percent(), old_rad, new_rad);
                *transform = transform.with_rotation(Quat::from_rotation_z(angle));
            }
        }
    }
}

fn pos_to_world(map: &Map, GridPos(x, y): GridPos) -> Vec3 {
    let height = match map.tile(GridPos(x, y)) {
        Place::UpperFloor => UPPER_FLOOR,
        Place::LowerFloor => LOWER_FLOOR,
        Place::Ramp(_) => (UPPER_FLOOR + LOWER_FLOOR) / 2.0,
        Place::Void => -1.0,
        _ => todo!(),
    };

    Vec3::new(
        x as f32 - map.width as f32 / 2.0,
        y as f32 - map.height as f32 / 2.0,
        height,
    )
}

fn dir_to_radians(dir: Direction) -> f32 {
    match dir {
        Direction::Up => 0.0,
        Direction::Down => std::f32::consts::PI,
        Direction::Left => std::f32::consts::PI * 0.5,
        Direction::Right => std::f32::consts::PI * 1.5,
    }
}

fn interpolate<T>(select: f32, start: T, end: T) -> T
where
    T: Mul<f32, Output = T>,
    T: Add<Output = T>,
{
    start * (1.0 - select) + end * select
}
