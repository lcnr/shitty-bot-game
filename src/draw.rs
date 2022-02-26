use std::collections::VecDeque;

use crate::map::EntityKind;
use crate::map::GridPos;
use crate::map::Map;
use crate::map::Place;
use crate::GameState;
use crate::Direction;
use bevy::prelude::*;

pub struct DrawPlugin;
impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(init_map_system)
            .add_system(update_map_system);
    }
}

pub struct DrawUpdates {
    data: VecDeque<Vec<()>>,
}

impl DrawUpdates {
    pub fn empty() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }
}

fn init_map_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
    map: Res<Map>,
    entities: Query<(Entity, &EntityKind, &GridPos)>,
) {
    if game_state.is_changed() && *game_state == GameState::Programming {
        // ok
    } else {
        return;
    }

    commands.spawn_bundle(DirectionalLightBundle {
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(map.width as f32, map.height as f32, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });

    let box_xy = shape::Box {
        min_x: 0.0,
        max_x: 1.0,
        min_y: 0.0,
        max_y: 1.0,
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
            match map.tile(x, y) {
                Place::UpperFloor => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: 0.5,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.5, 0.7, 0.5).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::LowerFloor => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: 0.3,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::Ramp(dir) => {
                    match dir {
                        Direction::Up => transform.rotate(Quat::from_rotation_z(0.5)),
                        Direction::Down => {}
                        Direction::Left => {}
                        Direction::Right => {}
                    }
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            min_z: -0.3,
                            max_z: 0.0,
                            max_y: 1.028039,
                            ..box_xy
                        })),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform,
                        ..Default::default()
                    });
                }
                Place::Void => {}
                Place::Wall => {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box {
                            max_z: 0.8,
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
}

fn update_map_system(events: ResMut<DrawUpdates>) {}
