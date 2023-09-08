use std::{
    fs::{read_to_string, write},
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use clap::Parser;
use futures_util::Future;
use log::{info, warn};
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use tokio::{
    select,
    sync::{broadcast, mpsc::unbounded_channel, RwLock},
    time::sleep,
};
use warp::Filter;

use game::Puzzle;

automod::dir!("src/");

use crate::{clients::ws_handler, puzzle_loader::PuzzleLoader, server_game_event::ServerGameEvent};

#[derive(Parser)]
struct Args {
    puzzle_json: Option<PathBuf>,
}

#[serde_as]
#[derive(Deserialize, Debug)]
struct Config {
    port: u16,
    #[serde_as(as = "DurationSeconds")]
    client_timeout: Duration,
    broadcast_channel_size: usize,

    backup_puzzle: bool,
    #[serde_as(as = "DurationSeconds")]
    puzzle_backup_interval: Duration,
    puzzle_backup_file: PathBuf,

    #[serde_as(as = "DurationSeconds")]
    completion_check_interval: Duration,
    #[serde_as(as = "DurationSeconds")]
    complete_wait_time: Duration,

    queue_file: PathBuf,

    tls_cert: PathBuf,
    tls_key: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Args::parse();

    let config_string = read_to_string("server_config.toml").unwrap();
    let config: Config = toml::from_str(&config_string).unwrap();
    info!("loaded config: {config:#?}");

    let mut puzzle_loader = PuzzleLoader::new(config.queue_file);

    let puzzle = if let Some(puzzle_json) = args.puzzle_json {
        info!("loading puzzle from {:?}", puzzle_json);
        Puzzle::deserialize(&read_to_string(&puzzle_json).unwrap()).unwrap()
    } else {
        puzzle_loader.next().unwrap()
    };

    let puzzle = Arc::new(RwLock::new(puzzle));

    let (event_input_tx, mut event_input_rx) = unbounded_channel::<ServerGameEvent>();
    let (event_output_tx, _) = broadcast::channel::<ServerGameEvent>(config.broadcast_channel_size);

    // route that serves up the client application
    let http_route = warp::fs::dir("dist");

    // client route that gives them a puzzle ref and channel handles
    let puzzle_clone = puzzle.clone();
    let event_output_tx_clone = event_output_tx.clone();
    let client_route = warp::path("client")
        .and(warp::ws())
        .and(warp::any().map(move || puzzle_clone.clone()))
        .and(warp::any().map(move || config.client_timeout))
        .and(warp::any().map(move || event_input_tx.clone()))
        .and(warp::any().map(move || event_output_tx_clone.subscribe()))
        .and_then(ws_handler);

    let routes = warp::get().and(http_route).or(client_route);
    let serve = warp::serve(routes);

    // don't use tls if dev
    let serve: Pin<Box<dyn Future<Output = ()>>> = if cfg!(debug_assertions) {
        warn!("starting server in dev mode without TLS");
        Box::pin(serve.run(([0, 0, 0, 0], config.port)))
    } else {
        Box::pin(
            serve
                .tls()
                .cert_path(config.tls_cert)
                .key_path(config.tls_key)
                .run(([0, 0, 0, 0], config.port)),
        )
    };

    // apply events to the puzzle and dispatch the generated events to clients
    let puzzle_clone = puzzle.clone();
    let event_handler = async move {
        while let Some(server_event) = event_input_rx.recv().await {
            let res_events = puzzle_clone
                .write()
                .await
                .apply_event(server_event.game_event);
            for res_event in res_events {
                let _ = event_output_tx.send(ServerGameEvent {
                    client_id: server_event.client_id,
                    game_event: res_event,
                });
            }
        }
    };

    let puzzle_clone = puzzle.clone();
    let puzzle_backup = async move {
        loop {
            sleep(config.puzzle_backup_interval).await;

            if config.backup_puzzle {
                let json = puzzle_clone.read().await.serialize();
                write(&config.puzzle_backup_file, json).unwrap();
            }
        }
    };

    let completion_handler = async move {
        while !puzzle.read().await.is_complete() {
            sleep(config.completion_check_interval).await;
        }

        info!("puzzle complete!");
        info!("shutting down server in {:?}...", config.complete_wait_time);

        puzzle_loader.pop_current();
        sleep(config.complete_wait_time).await;
    };

    select! {
        _ = serve => panic!(),
        _ = event_handler => panic!(),
        _ = puzzle_backup => panic!(),
        _ = completion_handler => (),
    };
}
