use std::sync::Arc;

use futures::future::join;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{
    broadcast::{self, Receiver},
    mpsc::{unbounded_channel, UnboundedSender},
    RwLock,
};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

use game::{AnyGameEvent, Puzzle};

//automod::dir!("src/");

const BROADCAST_CHANNEL_SIZE: usize = 10_000;

#[derive(Debug, Clone)]
struct ServerGameEvent {
    pub client_id: Uuid,
    pub event: AnyGameEvent,
}

#[tokio::main]
async fn main() {
    let puzzle = Arc::new(RwLock::new(load_puzzle().await));
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
    let serve = warp::serve(routes).run(([127, 0, 0, 1], 3030));

    // apply events to the puzzle and dispatch the generated events to clients
    let event_handler = async move {
        while let Some(event) = event_input_rx.recv().await {
            let res_events = puzzle.write().await.apply_event(event.event);
            for res_event in res_events {
                event_output_tx
                    .send(ServerGameEvent {
                        client_id: event.client_id,
                        event: res_event,
                    })
                    .unwrap();
            }
        }
    };

    join(serve, event_handler).await;
}

async fn load_puzzle() -> Puzzle {
    let response = reqwest::get("https://m.media-amazon.com/images/W/IMAGERENDERING_521856-T1/images/I/71tNdtNw70L._UF1000,1000_QL80_.jpg");
    let bytes = response.await.unwrap().bytes().await.unwrap();
    let image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::Jpeg)
        .unwrap()
        .to_rgba8();
    Puzzle::new(image, 36, true)
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
    let (mut ws_tx, mut ws_rx) = ws.split();

    // first, send the puzzle
    ws_tx
        .send(Message::text(&*puzzle.read().await.serialize()))
        .await
        .unwrap();

    // receive client events and forward them to server event handler
    let client_rx_handler = async move {
        while let Some(result) = ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(_) => break,
            };

            if msg.is_text() {
                let event = AnyGameEvent::deserialize(msg.to_str().unwrap());
                let event = ServerGameEvent { client_id, event };
                if event_tx.send(event).is_err() {
                    break;
                }
            } else {
                println!("{msg:?}");
            }
        }
        println!("client {client_id} disconnected");
    };

    // forward broadcasted events to client
    let client_tx_handler = async move {
        while let Ok(event) = event_rx.recv().await {
            // don't echo client events unless they're piece connected events
            // since those are always handled server-side first
            // to prevent non-deterministic connection logic due to rounding errors
            if event.client_id != client_id
                || matches!(event.event, AnyGameEvent::PieceConnected(_))
            {
                #[allow(clippy::collapsible_if)]
                if ws_tx
                    .send(Message::text(event.event.serialize()))
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
