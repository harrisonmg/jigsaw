use game::AnyGameEvent;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServerGameEvent {
    pub client_id: Uuid,
    pub game_event: AnyGameEvent,
}
