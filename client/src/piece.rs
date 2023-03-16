use std::slice::Iter;

use bevy::{prelude::*, utils::HashMap};

use game::{Piece, PieceIndex};

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
    sprite: SpriteBundle,
}

impl PieceBundle {
    pub fn from_piece(
        piece: &Piece,
        image_assets: &mut ResMut<Assets<Image>>,
        stack_pos: usize,
    ) -> Self {
        let sprite = piece.sprite_clone();
        Self {
            piece: PieceComponent {
                index: piece.index(),
                width: sprite.width() as f32,
                height: sprite.height() as f32,
                stack_pos,
            },
            sprite: SpriteBundle {
                texture: image_assets.add(sprite.into()),
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
