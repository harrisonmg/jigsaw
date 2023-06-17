use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use futures::future::join;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    sync::{
        broadcast::{self, Receiver},
        mpsc::{unbounded_channel, UnboundedSender},
        RwLock,
    },
    time::timeout,
};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

use game::{AnyGameEvent, PlayerDisconnectedEvent, Puzzle};

const BROADCAST_CHANNEL_SIZE: usize = 10_000;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60 * 10);

#[derive(Debug, Clone, Copy)]
struct ServerGameEvent {
    pub client_id: Uuid,
    pub game_event: AnyGameEvent,
}

#[derive(Parser)]
struct Args {
    image_url: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Args::parse();
    let puzzle = load_puzzle(args.image_url.as_str())
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Error loading puzzle\n\nIs \"{}\" the correct image URL?\n\nUnderlying error: {e}",
                args.image_url
            )
        });
    let puzzle = Arc::new(RwLock::new(puzzle));
    let (event_input_tx, mut event_input_rx) = unbounded_channel::<ServerGameEvent>();
    let (event_output_tx, _) = broadcast::channel::<ServerGameEvent>(BROADCAST_CHANNEL_SIZE);

    // add a client route that gives them a puzzle ref and channel handles
    let puzzle_clone = puzzle.clone();
    let event_output_tx_clone = event_output_tx.clone();
    let routes = warp::path("client")
        .and(warp::ws())
        .and(warp::any().map(move || puzzle_clone.clone()))
        .and(warp::any().map(move || event_input_tx.clone()))
        .and(warp::any().map(move || event_output_tx_clone.subscribe()))
        .and_then(ws_handler);

    // serve that shit up
    let serve = warp::serve(routes).run(([0, 0, 0, 0], 3030));

    // apply events to the puzzle and dispatch the generated events to clients
    let event_handler = async move {
        while let Some(server_event) = event_input_rx.recv().await {
            let res_events = puzzle.write().await.apply_event(server_event.game_event);
            for res_event in res_events {
                event_output_tx
                    .send(ServerGameEvent {
                        client_id: server_event.client_id,
                        game_event: res_event,
                    })
                    .unwrap();
            }
        }
    };

    join(serve, event_handler).await;
}

async fn load_puzzle(image_url: &str) -> Result<Puzzle> {
    let response = reqwest::get(image_url);
    let bytes = response.await?.bytes().await?;
    let image =
        image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::Jpeg)?.to_rgba8();
    Ok(Puzzle::new(image, 1000, true))
}

async fn ws_handler(
    ws: warp::ws::Ws,
    puzzle: Arc<RwLock<Puzzle>>,
    event_tx: UnboundedSender<ServerGameEvent>,
    event_rx: Receiver<ServerGameEvent>,
) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |warp_ws| client_handler(warp_ws, puzzle, event_tx, event_rx)))
}

async fn client_handler(
    ws: WebSocket,
    puzzle: Arc<RwLock<Puzzle>>,
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
            if let Ok(item) = timeout(CLIENT_TIMEOUT, ws_rx.next()).await {
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
            // since those are always handled server-side first
            // to prevent non-deterministic connection logic due to rounding errors
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
