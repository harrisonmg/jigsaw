use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

/// A rectangle on the `XY` plane.
#[derive(Debug, Copy, Clone)]
pub struct BetterQuad {
    /// Full width and height of the rectangle.
    pub size: Vec2,
    /// Origin point of the quad.
    pub origin: Vec2,
    /// Horizontally-flip the texture coordinates of the resulting mesh.
    pub flip: bool,
}

impl BetterQuad {
    pub fn new(size: Vec2, origin: Vec2) -> Self {
        Self {
            size,
            origin,
            flip: false,
        }
    }

    pub fn flipped(size: Vec2, origin: Vec2) -> Self {
        Self {
            size,
            origin,
            flip: true,
        }
    }
}

impl From<BetterQuad> for Mesh {
    fn from(quad: BetterQuad) -> Self {
        let (u_left, u_right) = if quad.flip { (1.0, 0.0) } else { (0.0, 1.0) };
        let vertices = [
            (
                [-quad.origin.x, -quad.origin.y, 0.0],
                [0.0, 0.0, 1.0],
                [u_left, 1.0],
            ),
            (
                [-quad.origin.x, -quad.origin.y + quad.size.y, 0.0],
                [0.0, 0.0, 1.0],
                [u_left, 0.0],
            ),
            (
                [
                    -quad.origin.x + quad.size.x,
                    -quad.origin.y + quad.size.y,
                    0.0,
                ],
                [0.0, 0.0, 1.0],
                [u_right, 0.0],
            ),
            (
                [-quad.origin.x + quad.size.x, -quad.origin.y, 0.0],
                [0.0, 0.0, 1.0],
                [u_right, 1.0],
            ),
        ];

        let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::all(),
        );
        mesh.insert_indices(indices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
