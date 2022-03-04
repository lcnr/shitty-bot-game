use std::collections::VecDeque;
use std::ops::Add;
use std::ops::Deref;
use std::ops::Mul;

use crate::bot::BotData;
use crate::map::EntityKind;
use crate::map::GridPos;
use crate::map::Level;
use crate::map::Map;
use crate::map::Place;
use crate::util::StateLocal;
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

pub fn init_timer(world: &mut World) {
    world.insert_resource(DrawTimer(Timer::from_seconds(0.5, false)));
}

pub fn init_map_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
    entities: Query<(Entity, &EntityKind, &GridPos)>,
    robots: Query<&BotData>,
) {
    commands
        .spawn_bundle(DirectionalLightBundle {
            transform: Transform::from_xyz(1.7, 10.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(StateLocal);
    let viewing_pos = Vec3::new(7.0, 17.0, 10.0);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(viewing_pos)
                .looking_at(Vec3::ZERO, Vec3::Y)
                .with_translation(
                    viewing_pos
                        + (Vec3::X * level.map.width as f32)
                        + (Vec3::Y * level.map.height as f32 / 3.0),
                ),
            ..Default::default()
        })
        .insert(StateLocal);

    let box_xy = shape::Box {
        min_x: -0.5,
        max_x: 0.5,
        min_y: 0.0,
        max_y: 1.0,
        min_z: -0.5,
        max_z: 0.5,
    };

    for x in 0..level.map.width {
        for y in 0..level.map.height {
            let transform = Transform::from_xyz(
                x as f32 - level.map.width as f32 / 2.0,
                0.0,
                y as f32 - level.map.height as f32 / 2.0,
            );
            match level.map.tile(GridPos(x, y)) {
                Place::UpperFloor => {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box {
                                max_y: UPPER_FLOOR,
                                ..box_xy
                            })),
                            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                            transform,
                            ..Default::default()
                        })
                        .insert(StateLocal);
                }
                Place::LowerFloor => {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box {
                                max_y: LOWER_FLOOR,
                                ..box_xy
                            })),
                            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                            transform,
                            ..Default::default()
                        })
                        .insert(StateLocal);
                }
                Place::Ramp(dir) => {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(mesh::slope_mesh(dir)),
                            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                            transform: transform
                                .looking_at(transform.translation - Vec3::Y, Vec3::Z),
                            ..Default::default()
                        })
                        .insert(StateLocal);
                }
                Place::Void => {}
                Place::Wall => {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box {
                                max_y: 0.9,
                                ..box_xy
                            })),
                            material: materials.add(Color::rgb(0.7, 0.9, 0.7).into()),
                            transform,
                            ..Default::default()
                        })
                        .insert(StateLocal);
                }
                Place::Exit => {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box {
                                max_y: 1.0,
                                ..box_xy
                            })),
                            material: materials.add(Color::rgb(0.1, 0.9, 0.1).into()),
                            transform,
                            ..Default::default()
                        })
                        .insert(StateLocal);
                }
            }
        }
    }

    for (entity, kind, &position) in entities.iter() {
        let transform = Transform::from_translation(pos_to_world(&level.map, position));

        match kind {
            EntityKind::Robot => {
                let state = robots.get(entity).expect("bot without bot state");
                commands.get_or_spawn(entity).insert_bundle(PbrBundle {
                    mesh: meshes.add(mesh::robot_mesh()),
                    material: materials.add(Color::rgb(0.25, 0.12, 0.1).into()),
                    transform: transform.with_rotation(
                        Quat::from_rotation_y(dir_to_radians(state.start_dir))
                            * Quat::from_rotation_x(std::f32::consts::PI * 1.5),
                    ),
                    ..Default::default()
                });
            }
            EntityKind::Box => {
                commands.get_or_spawn(entity).insert_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box {
                        min_x: -0.4,
                        max_x: 0.4,
                        min_y: 0.0,
                        max_y: 0.8,
                        min_z: -0.4,
                        max_z: 0.4,
                    })),
                    material: materials.add(Color::rgb(0.25, 0.12, 0.1).into()),
                    transform,
                    ..Default::default()
                });
            }
        }
    }
}

pub struct DrawTimer(pub Timer);
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
    level: Res<Level>,
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
                let old_pos = pos_to_world(&level.map, from);
                let new_pos = pos_to_world(&level.map, to);
                let position = interpolate(timer.percent(), old_pos, new_pos);
                *transform = transform.with_translation(position);
            }
            Step::MoveFail => {}
            Step::UpdateDir(old, new) => {
                let mut transform = transforms.get_mut(entity).expect("sus step");
                let old_rad = dir_to_radians(old);
                let new_rad = dir_to_radians(new);
                let old_rot = Quat::from_rotation_y(old_rad)
                    * Quat::from_rotation_x(dir_to_radians(Direction::Right));
                let new_rot = Quat::from_rotation_y(new_rad)
                    * Quat::from_rotation_x(dir_to_radians(Direction::Right));
                if old_rot.dot(new_rot) < 0. {
                    transform.rotation = (-old_rot).slerp(new_rot, timer.percent());
                } else {
                    transform.rotation = old_rot.slerp(new_rot, timer.percent());
                }
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
        Place::Exit => LOWER_FLOOR,
        _ => todo!(),
    };

    Vec3::new(
        x as f32 - map.width as f32 / 2.0,
        height,
        y as f32 - map.height as f32 / 2.0,
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
