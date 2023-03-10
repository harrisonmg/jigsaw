use crate::{Piece, PieceIndex};

#[derive(Debug)]
pub struct PieceMoveEvent {
    pub index: PieceIndex,

    // global absolute pose
    pub x: f32,
    pub y: f32,
    pub rotation: f32, // radians around Z axis
}

impl PieceMoveEvent {
    pub(crate) fn from_piece(value: &Piece) -> Self {
        Self {
            index: value.index(),
            x: value.transform.translation.x,
            y: value.transform.translation.y,
            rotation: value.transform.rotation.xyz().z,
        }
    }
}
