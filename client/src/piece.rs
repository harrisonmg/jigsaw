use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};

use game::{Piece, PieceIndex};

use crate::material::PieceMaterial;

#[derive(Component)]
pub struct PieceComponent {
    index: PieceIndex,
    width: f32,
    height: f32,
    pub stack_pos: usize,
}

impl PieceComponent {
    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
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
        let piece_component = PieceComponent {
            index: piece.index(),
            width: sprite.width() as f32,
            height: sprite.height() as f32,
            stack_pos,
        };
        let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            sprite.width() as f32,
            sprite.height() as f32,
        ))));
        let material = materials.add(PieceMaterial {
            texture: image_assets.add(sprite.into()),
        });
        Self {
            piece: piece_component,
            mesh_bundle: MaterialMesh2dBundle {
                mesh: mesh.into(),
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
