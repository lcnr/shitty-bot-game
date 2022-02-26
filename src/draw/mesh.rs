use bevy::render::mesh::Indices;
use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use super::{LOWER_FLOOR, UPPER_FLOOR};
use crate::Direction;

pub fn slope_mesh(dir: Direction) -> Mesh {
    let (x0y0, x1y0, x1y1, x0y1) = match dir {
        Direction::Up => (LOWER_FLOOR, LOWER_FLOOR, UPPER_FLOOR, UPPER_FLOOR),
        Direction::Down => (UPPER_FLOOR, UPPER_FLOOR, LOWER_FLOOR, LOWER_FLOOR),
        Direction::Left => (LOWER_FLOOR, UPPER_FLOOR, UPPER_FLOOR, LOWER_FLOOR),
        Direction::Right => (UPPER_FLOOR, LOWER_FLOOR, LOWER_FLOOR, UPPER_FLOOR),
    };

    let vertices = &[
        // Top
        ([0.0, 0.0, x0y0], [0., 0., 1.0], [0., 0.]),
        ([1.0, 0.0, x1y0], [0., 0., 1.0], [1.0, 0.]),
        ([1.0, 1.0, x1y1], [0., 0., 1.0], [1.0, 1.0]),
        ([0.0, 1.0, x0y1], [0., 0., 1.0], [0., 1.0]),
        // Bottom
        ([0.0, 1.0, 0.0], [0., 0., -1.0], [1.0, 0.]),
        ([1.0, 1.0, 0.0], [0., 0., -1.0], [0., 0.]),
        ([1.0, 0.0, 0.0], [0., 0., -1.0], [0., 1.0]),
        ([0.0, 0.0, 0.0], [0., 0., -1.0], [1.0, 1.0]),
        // Right
        ([1.0, 0.0, 0.0], [1.0, 0., 0.], [0., 0.]),
        ([1.0, 1.0, 0.0], [1.0, 0., 0.], [1.0, 0.]),
        ([1.0, 1.0, x1y1], [1.0, 0., 0.], [1.0, 1.0]),
        ([1.0, 0.0, x1y0], [1.0, 0., 0.], [0., 1.0]),
        // Left
        ([0.0, 0.0, x0y0], [-1.0, 0., 0.], [1.0, 0.]),
        ([0.0, 1.0, x0y1], [-1.0, 0., 0.], [0., 0.]),
        ([0.0, 1.0, 0.0], [-1.0, 0., 0.], [0., 1.0]),
        ([0.0, 0.0, 0.0], [-1.0, 0., 0.], [1.0, 1.0]),
        // Front
        ([1.0, 1.0, 0.0], [0., 1.0, 0.], [1.0, 0.]),
        ([0.0, 1.0, 0.0], [0., 1.0, 0.], [0., 0.]),
        ([0.0, 1.0, x0y1], [0., 1.0, 0.], [0., 1.0]),
        ([1.0, 1.0, x1y1], [0., 1.0, 0.], [1.0, 1.0]),
        // Back
        ([1.0, 0.0, x1y0], [0., -1.0, 0.], [0., 0.]),
        ([0.0, 0.0, x0y0], [0., -1.0, 0.], [1.0, 0.]),
        ([0.0, 0.0, 0.0], [0., -1.0, 0.], [1.0, 1.0]),
        ([1.0, 0.0, 0.0], [0., -1.0, 0.], [0., 1.0]),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);

    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
