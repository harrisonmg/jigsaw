use bevy::{
    prelude::*, render::mesh::VertexAttributeValues, sprite::MaterialMesh2dBundle, utils::HashMap,
};

use game::{Piece, PieceIndex};

use crate::material::PieceMaterial;

#[derive(Component)]
pub struct PieceComponent {
    index: PieceIndex,
    pub stack_pos: usize,
}

impl PieceComponent {
    pub fn index(&self) -> PieceIndex {
        self.index
    }
}

#[derive(Bundle)]
pub struct PieceBundle {
    pub piece: PieceComponent,

    #[bundle]
    mesh_bundle: MaterialMesh2dBundle<PieceMaterial>,
}

impl PieceBundle {
    pub fn new(
        piece: &Piece,
        stack_pos: usize,
        image_assets: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<PieceMaterial>>,
    ) -> Self {
        let sprite = piece.sprite_clone();

        let sprite_width = sprite.width() as f32;
        let sprite_height = sprite.height() as f32;

        let sprite_origin_x = piece.sprite_origin_x() as f32 / sprite_width;
        let sprite_origin_y = piece.sprite_origin_y() as f32 / sprite_height;

        let piece_component = PieceComponent {
            index: piece.index(),
            stack_pos,
        };

        let mut mesh = Mesh::from(shape::Quad::new(Vec2::new(sprite_width, sprite_height)));
        let x_offset = 0.5 - sprite_origin_x;
        let y_offset = 0.5 - sprite_origin_y;
        let new_vertices = if let VertexAttributeValues::Float32x3(vertices) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            vertices
                .iter()
                .map(|vertex| {
                    let mut points = vertex.clone();
                    points[0] += x_offset;
                    points[1] += y_offset;
                    points
                })
                .collect::<Vec<[f32; 3]>>()
        } else {
            panic!();
        };
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_vertices);

        let mesh_handle = meshes.add(mesh);
        let material = materials.add(PieceMaterial {
            texture: image_assets.add(sprite.into()),
            sprite_origin: Vec2 {
                x: sprite_origin_x,
                y: sprite_origin_y,
            },
        });
        Self {
            piece: piece_component,
            mesh_bundle: MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material,
                transform: piece.transform(),
                ..Default::default()
            },
        }
    }
}

#[derive(Resource)]
pub struct PieceMap(pub HashMap<PieceIndex, Entity>);

#[derive(Resource)]
pub struct PieceStack(pub Vec<Entity>);

impl PieceStack {
    pub fn put_on_top(&mut self, piece: &mut PieceComponent, entity: Entity) {
        piece.stack_pos = self.0.len();
        self.0.push(entity);
    }
}

#[derive(Resource)]
pub struct HeldPiece {
    pub index: PieceIndex,
    pub cursor_position: Vec2,
}
