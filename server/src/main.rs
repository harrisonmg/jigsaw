use futures_util::{SinkExt, StreamExt};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use warp::{ws::Message, Filter};

use game::Puzzle;

automod::dir!("src/");

#[tokio::main]
async fn main() {
    let response = reqwest::get("https://m.media-amazon.com/images/W/IMAGERENDERING_521856-T1/images/I/71tNdtNw70L._UF1000,1000_QL80_.jpg");
    let bytes = response.await.unwrap().bytes().await.unwrap();
    let image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::Jpeg)
        .unwrap()
        .to_rgba8();
    let puzzle = Puzzle::new(image, 36, true);

    let mut buf = Vec::new();
    puzzle.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let routes = warp::path("echo")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let buf = buf.clone();
            ws.on_upgrade(move |websocket| {
                let (mut tx, _rx) = websocket.split();
                tx.send(Message::binary(buf.clone()));
                futures::future::ready(())
            })
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
