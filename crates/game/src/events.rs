use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Piece, PieceIndex, Player};

pub trait GameEvent {
    fn serialize(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AnyGameEvent {
    PieceMoved(PieceMovedEvent),
    PiecePickedUp(PiecePickedUpEvent),
    PiecePutDown(PiecePutDownEvent),
    PieceConnected(PieceConnectedEvent),
    PlayerConnected(PlayerConnectedEvent),
    CursorMoved(CursorMovedEvent),
}

impl AnyGameEvent {
    pub fn deserialize(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PieceMovedEvent {
    pub index: PieceIndex,
    pub x: f32,
    pub y: f32,
}

impl GameEvent for PieceMovedEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PieceMoved(self.clone())).unwrap()
    }
}

impl From<&Piece> for PieceMovedEvent {
    fn from(value: &Piece) -> Self {
        Self {
            index: value.index(),
            x: value.transform.translation.x,
            y: value.transform.translation.y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PiecePickedUpEvent {
    pub index: PieceIndex,
}

impl GameEvent for PiecePickedUpEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PiecePickedUp(self.clone())).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PiecePutDownEvent {
    pub index: PieceIndex,
}

impl GameEvent for PiecePutDownEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PiecePutDown(self.clone())).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PieceConnectedEvent {
    pub index: PieceIndex,
}

impl GameEvent for PieceConnectedEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PieceConnected(self.clone())).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CursorMovedEvent {
    pub player_id: Uuid,
    pub x: f32,
    pub y: f32,
}

impl GameEvent for CursorMovedEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::CursorMoved(self.clone())).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerConnectedEvent {
    pub player: Player,
}

impl GameEvent for PlayerConnectedEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PlayerConnected(self.clone())).unwrap()
    }
}
