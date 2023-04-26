use bevy::prelude::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PieceIndex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub client_id: Uuid,
    pub held_piece: Option<PieceIndex>,
    //pub cursor: Cursor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub color: Color,
    pub x: f32,
    pub y: f32,
}
