use anyhow::Result;
use bevy::prelude::Event;
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
    PlayerCursorMoved(PlayerCursorMovedEvent),
    PlayerDisconnected(PlayerDisconnectedEvent),
}

impl AnyGameEvent {
    pub fn deserialize(value: &str) -> Result<Self> {
        serde_json::from_str(value).map_err(anyhow::Error::from)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn add_player_id(&mut self, id: Uuid) {
        match self {
            AnyGameEvent::PiecePickedUp(ref mut event) => event.player_id = Some(id),
            AnyGameEvent::PiecePutDown(ref mut event) => event.player_id = Some(id),
            AnyGameEvent::PlayerCursorMoved(ref mut event) => event.player_id = Some(id),
            _ => (),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PieceMovedEvent {
    pub index: PieceIndex,
    pub x: f32,
    pub y: f32,
}

impl GameEvent for PieceMovedEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&AnyGameEvent::PieceMoved(*self)).unwrap()
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PiecePickedUpEvent {
    pub player_id: Option<Uuid>,
    pub index: PieceIndex,
}

impl GameEvent for PiecePickedUpEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PiecePickedUp(*self).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PiecePutDownEvent {
    pub player_id: Option<Uuid>,
    pub index: PieceIndex,
}

impl GameEvent for PiecePutDownEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PiecePutDown(*self).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PieceConnectionEvent {
    pub index: PieceIndex,
}

impl GameEvent for PieceConnectionEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PieceConnection(*self).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PlayerCursorMovedEvent {
    pub player_id: Option<Uuid>,
    pub cursor: Cursor,
}

impl GameEvent for PlayerCursorMovedEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PlayerCursorMoved(*self).serialize()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Event)]
pub struct PlayerDisconnectedEvent {
    pub player_id: Uuid,
}

impl GameEvent for PlayerDisconnectedEvent {
    fn serialize(&self) -> String {
        AnyGameEvent::PlayerDisconnected(*self).serialize()
    }
}
