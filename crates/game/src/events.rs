use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Cursor, Piece, PieceIndex};

pub trait GameEvent {
    fn serialize(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum AnyGameEvent {
    PieceMoved(PieceMovedEvent),
    PiecePickedUp(PiecePickedUpEvent),
    PiecePutDown(PiecePutDownEvent),
    PieceConnection(PieceConnectionEvent),
    PlayerConnected(PlayerConnectedEvent),
    PlayerDisconnected(PlayerDisconnectedEvent),
    CursorMoved(CursorMovedEvent),
}

impl AnyGameEvent {
    pub fn deserialize(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PiecePickedUpEvent {
    pub player_id: Uuid,
    pub index: PieceIndex,
}

impl GameEvent for PiecePickedUpEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PiecePickedUp(self.clone()).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PiecePutDownEvent {
    pub player_id: Uuid,
    pub index: PieceIndex,
}

impl GameEvent for PiecePutDownEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PiecePutDown(self.clone()).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PieceConnectionEvent {
    pub index: PieceIndex,
}

impl GameEvent for PieceConnectionEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PieceConnection(self.clone()).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CursorMovedEvent {
    pub player_id: Uuid,
    pub x: f32,
    pub y: f32,
}

impl GameEvent for CursorMovedEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::CursorMoved(self.clone()).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerConnectedEvent {
    pub player_id: Uuid,
    pub cursor: Cursor,
}

impl GameEvent for PlayerConnectedEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PlayerConnected(self.clone()).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerDisconnectedEvent {
    pub player_id: Uuid,
}

impl GameEvent for PlayerDisconnectedEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PlayerDisconnected(self.clone()).serialize()
    }
}
