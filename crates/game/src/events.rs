use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Piece, PieceIndex, Player};

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEvent {
    PieceMoved(PieceMovedEvent),
    PiecePickedUp(PiecePickedUpEvent),
    PiecePutDown(PiecePutDownEvent),
    PieceConnected(PieceConnectedEvent),
    CursorMoved(CursorMovedEvent),
    PlayerConnected(PlayerConnectedEvent),
}

impl From<&str> for GameEvent {
    fn from(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }
}

impl From<&GameEvent> for String {
    fn from(value: &GameEvent) -> Self {
        serde_json::to_string(value).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieceMovedEvent {
    pub index: PieceIndex,

    // global absolute pose
    pub x: f32,
    pub y: f32,
}

impl From<PieceMovedEvent> for GameEvent {
    fn from(value: PieceMovedEvent) -> Self {
        GameEvent::PieceMoved(value)
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PiecePickedUpEvent {
    pub index: PieceIndex,
}

impl From<PiecePickedUpEvent> for GameEvent {
    fn from(value: PiecePickedUpEvent) -> Self {
        GameEvent::PiecePickedUp(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PiecePutDownEvent {
    pub index: PieceIndex,
}

impl From<PiecePutDownEvent> for GameEvent {
    fn from(value: PiecePutDownEvent) -> Self {
        GameEvent::PiecePutDown(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieceConnectedEvent {
    pub index: PieceIndex,
}

impl From<PieceConnectedEvent> for GameEvent {
    fn from(value: PieceConnectedEvent) -> Self {
        GameEvent::PieceConnected(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CursorMovedEvent {
    pub player_id: Uuid,

    // global absolute pose
    pub x: f32,
    pub y: f32,
}

impl From<CursorMovedEvent> for GameEvent {
    fn from(value: CursorMovedEvent) -> Self {
        GameEvent::CursorMoved(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerConnectedEvent {
    pub player: Player,
}

impl From<PlayerConnectedEvent> for GameEvent {
    fn from(value: PlayerConnectedEvent) -> Self {
        GameEvent::PlayerConnected(value)
    }
}
