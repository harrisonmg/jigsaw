use std::{sync::Arc, time::Duration};

use anyhow::Result;
use futures::future::join;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    sync::{broadcast::Receiver, mpsc::UnboundedSender, RwLock},
    time::timeout,
};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Rejection, Reply,
};

use game::{AnyGameEvent, PlayerDisconnectedEvent, Puzzle};

use crate::server_game_event::ServerGameEvent;

pub async fn ws_handler(
    ws: warp::ws::Ws,
    puzzle: Arc<RwLock<Puzzle>>,
    client_timeout: Duration,
    event_tx: UnboundedSender<ServerGameEvent>,
    event_rx: Receiver<ServerGameEvent>,
) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |warp_ws| {
        client_handler(warp_ws, puzzle, client_timeout, event_tx, event_rx)
    }))
}

pub async fn client_handler(
    ws: WebSocket,
    puzzle: Arc<RwLock<Puzzle>>,
    client_timeout: Duration,
    event_tx: UnboundedSender<ServerGameEvent>,
    mut event_rx: Receiver<ServerGameEvent>,
) {
    let client_id = Uuid::new_v4();

    info!("client {client_id} connected");

    let (mut ws_tx, mut ws_rx) = ws.split();

    // first, send the puzzle
    let msg = Message::text(&*puzzle.read().await.serialize());

    if ws_tx.send(msg).await.is_err() {
        info!("client {client_id} disconnected");
        return;
    }

    // receive client events and forward them to server event handler
    let client_rx_handler = async move {
        loop {
            if let Ok(item) = timeout(client_timeout, ws_rx.next()).await {
                let res = match item {
                    Some(res) => res,
                    None => {
                        error!("no item received from client {client_id}");
                        break;
                    }
                };

                let msg = match res {
                    Ok(msg) => msg,
                    Err(_) => break,
                };

                if msg.is_text() {
                    if let Ok(mut game_event) = AnyGameEvent::deserialize(msg.to_str().unwrap()) {
                        game_event.add_player_id(client_id);

                        let server_event = ServerGameEvent {
                            client_id,
                            game_event,
                        };

                        if let Err(e) = event_tx.send(server_event) {
                            error!(
                            "error sending event to server model in client {client_id} task: {e}"
                        );
                            break;
                        }
                    } else {
                        error!("malformed message from client {client_id}: {msg:?}");
                        break;
                    }
                } else {
                    if !msg.is_close() {
                        error!("unhandled message from client {client_id}: {msg:?}");
                    }
                    break;
                }
            } else {
                info!("client {client_id} timed out");
                break;
            }
        }

        let res = event_tx.send(ServerGameEvent {
            client_id,
            game_event: AnyGameEvent::PlayerDisconnected(PlayerDisconnectedEvent {
                player_id: client_id,
            }),
        });

        match res {
            Ok(()) => info!("client {client_id} disconnected"),
            Err(e) => error!("error sending event to server model in client {client_id} task: {e}"),
        }
    };

    // forward broadcasted events to client
    let client_tx_handler = async move {
        while let Ok(event) = event_rx.recv().await {
            if event.client_id == client_id
                && matches!(event.game_event, AnyGameEvent::PlayerDisconnected(_))
            {
                break;
            }

            // don't echo client events unless they're piece connection events
            if event.client_id != client_id
                || matches!(event.game_event, AnyGameEvent::PieceConnection(_))
            {
                #[allow(clippy::collapsible_if)]
                if ws_tx
                    .send(Message::text(event.game_event.serialize()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    };

    join(client_rx_handler, client_tx_handler).await;
}
