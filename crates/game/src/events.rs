use serde::{Deserialize, Serialize};

use crate::{Piece, PieceIndex};

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEvent {
    PieceMoved(PieceMovedEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieceMovedEvent {
    pub index: PieceIndex,

    // global absolute pose
    pub x: f32,
    pub y: f32,
}

impl PieceMovedEvent {
    pub(crate) fn from_piece(value: &Piece) -> Self {
        Self {
            index: value.index(),
            x: value.transform.translation.x,
            y: value.transform.translation.y,
        }
    }
}
