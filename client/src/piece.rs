use bevy::prelude::*;

use game::{Piece, PieceIndex};

#[derive(Component)]
pub struct PieceComponent {
    index: PieceIndex,
    width: f32,
    height: f32,
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
    piece: PieceComponent,

    #[bundle]
    sprite: SpriteBundle,
}

impl PieceBundle {
    pub fn from_piece(piece: &Piece, image_assets: &mut ResMut<Assets<Image>>) -> Self {
        let sprite = piece.sprite_clone();
        Self {
            piece: PieceComponent {
                index: piece.index(),
                width: sprite.width() as f32,
                height: sprite.height() as f32,
            },
            sprite: SpriteBundle {
                texture: image_assets.add(sprite.into()),
                transform: piece.transform(),
                ..Default::default()
            },
        }
    }
}
