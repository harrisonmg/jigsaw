use std::sync::Arc;

use futures::{future::ready, FutureExt};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

use game::Puzzle;

//automod::dir!("src/");

#[tokio::main]
async fn main() {
    let puzzle = Arc::new(RwLock::new(load_puzzle().await));

    let routes = warp::path("client")
        .and(warp::ws())
        .and(warp::any().map(move || puzzle.clone()))
        .and_then(ws_handler);

    let serve = warp::serve(routes).run(([127, 0, 0, 1], 3030));
    serve.await;
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
) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |warp_ws| client_handler(warp_ws, puzzle)))
}

async fn client_handler(ws: WebSocket, puzzle: Arc<RwLock<Puzzle>>) {
    let (mut tx, _rx) = ws.split();
    tx.send(Message::text(&*puzzle.read().await)).await.unwrap();
}
