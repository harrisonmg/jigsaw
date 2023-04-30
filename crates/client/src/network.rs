use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_util::future::join;
use futures_util::{SinkExt, StreamExt};
use game::{AnyGameEvent, GameEvent, PieceMovedEvent, Puzzle};
use ws_stream_wasm::{WsMessage, WsMeta};

use crate::states::AppState;
use crate::worker::Worker;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), spawn_network_io_task)
            .add_systems(Update, load_puzzle.run_if(in_state(AppState::Loading)))
            .add_systems(Update, event_io.run_if(in_state(AppState::Playing)));
    }
}

type NetworkIO = Worker<String, String>;

fn spawn_network_io_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let io = NetworkIO::spawn(thread_pool, |mut client_rx, client_tx| async move {
        let (_, ws_io) = WsMeta::connect("ws://127.0.0.1:3030/client", None)
            .await
            .unwrap();
        let (mut ws_tx, mut ws_rx) = ws_io.split();

        let net_rx_handler = async move {
            while let Some(WsMessage::Text(msg)) = ws_rx.next().await {
                client_tx.send(msg).unwrap();
            }
        };

        let net_tx_handler = async move {
            while let Some(msg) = client_rx.recv().await {
                ws_tx.send(WsMessage::Text(msg)).await.unwrap();
            }
        };

        join(net_rx_handler, net_tx_handler).await;
    });
    commands.insert_resource(io);
}

fn load_puzzle(
    mut commands: Commands,
    mut network_io: ResMut<NetworkIO>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    while let Ok(msg) = network_io.output.try_recv() {
        let puzzle = Puzzle::deserialize(msg.as_str());
        commands.insert_resource(puzzle);
        next_state.set(AppState::Setup);
    }
}

fn event_io(
    mut piece_moved_events: ResMut<Events<PieceMovedEvent>>,
    mut piece_moved_event_reader: Local<ManualEventReader<PieceMovedEvent>>,
    mut network_io: ResMut<NetworkIO>,
    mut puzzle: ResMut<Puzzle>,
) {
    use AnyGameEvent::*;

    // forward all events generated by the client to the server
    for event in piece_moved_event_reader.iter(&piece_moved_events) {
        network_io.input.send(event.serialize()).unwrap();
    }

    // receive events from the server and apply them to the local puzzle instance
    let mut new_events = Vec::new();
    while let Ok(msg) = network_io.output.try_recv() {
        let event = AnyGameEvent::deserialize(msg.as_str());
        new_events.extend(puzzle.apply_event(event));
    }

    // dispatch new events out to bevy
    for event in new_events {
        match event {
            PieceMoved(event) => piece_moved_events.send(event),
            PiecePickedUp(_event) => (),
            PiecePutDown(_event) => (),
            PieceConnected(_event) => (),
            PlayerConnected(_event) => (),
            CursorMoved(_event) => (),
        }
    }

    // consume all the events we just dispatched so we don't forward them back out next frame
    piece_moved_event_reader.clear(&piece_moved_events);
}
