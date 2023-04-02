use crate::{Piece, PieceIndex};

#[derive(Debug)]
pub struct PieceMoved {
    pub index: PieceIndex,

    // global absolute pose
    pub x: f32,
    pub y: f32,
}

impl PieceMoved {
    pub(crate) fn from_piece(value: &Piece) -> Self {
        Self {
            index: value.index(),
            x: value.transform.translation.x,
            y: value.transform.translation.y,
        }
    }
}
