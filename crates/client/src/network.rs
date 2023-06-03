use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_util::future::join;
use futures_util::{select, FutureExt, SinkExt, StreamExt};
use game::{
    AnyGameEvent, GameEvent, PieceConnectionEvent, PieceMovedEvent, PiecePickedUpEvent,
    PiecePutDownEvent, PlayerCursorMovedEvent, PlayerDisconnectedEvent, Puzzle,
};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot;
use ws_stream_wasm::{WsMessage, WsMeta};

use crate::states::AppState;
use crate::worker::Worker;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PieceMovedEvent>()
            .add_event::<PiecePickedUpEvent>()
            .add_event::<PiecePutDownEvent>()
            .add_event::<PieceConnectionEvent>()
            .add_event::<PlayerCursorMovedEvent>()
            .add_event::<PlayerDisconnectedEvent>()
            .add_systems(OnEnter(AppState::Loading), spawn_network_io_task)
            .add_systems(Update, load_puzzle.run_if(in_state(AppState::Loading)))
            .add_systems(Update, event_io.run_if(in_state(AppState::Playing)));
    }
}

type NetworkIO = Worker<String, String>;

fn spawn_network_io_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let io = NetworkIO::spawn(thread_pool, |mut client_rx, client_tx| async move {
        let ws_io = match WsMeta::connect("ws://71.233.100.144:3030/client", None).await {
            Ok((_, ws_io)) => ws_io,
            Err(_) => {
                return;
            }
        };

        let (mut ws_tx, mut ws_rx) = ws_io.split();
        let (dc_tx, dc_rx) = oneshot::channel();

        let net_rx_handler = async move {
            let mut disconnect = dc_rx.fuse();
            loop {
                select! {
                    _ = disconnect => {
                        break;
                    },
                    res = ws_rx.next().fuse() => match res {
                        None => break,
                        Some(msg) => match msg {
                            WsMessage::Text(msg) => client_tx.send(msg).unwrap(),
                            WsMessage::Binary(msg) => warn!("strange message received from server: {msg:?}"),
                        }
                    },
                }
            }
        };

        let net_tx_handler = async move {
            while let Some(msg) = client_rx.recv().await {
                if ws_tx.send(WsMessage::Text(msg)).await.is_err() {
                    break;
                }
            }
            let _ = dc_tx.send(());
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
    match network_io.output.try_recv() {
        Ok(msg) => {
            if let Ok(puzzle) = Puzzle::deserialize(msg.as_str()) {
                commands.insert_resource(puzzle);
                next_state.set(AppState::Setup);
            }
        }
        Err(e) => match e {
            TryRecvError::Empty => (),
            TryRecvError::Disconnected => (), // TODO something here? hard to handle when not even the camera is loaded
                                              // Maybe the loading screen will help. Could be a good reason to do it in engine.
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn event_io(
    mut piece_moved_events: ResMut<Events<PieceMovedEvent>>,
    mut piece_moved_reader: Local<ManualEventReader<PieceMovedEvent>>,

    mut piece_picked_up_events: ResMut<Events<PiecePickedUpEvent>>,
    mut piece_picked_up_reader: Local<ManualEventReader<PiecePickedUpEvent>>,

    mut piece_put_down_events: ResMut<Events<PiecePutDownEvent>>,
    mut piece_put_down_reader: Local<ManualEventReader<PiecePutDownEvent>>,

    mut piece_connection_events: ResMut<Events<PieceConnectionEvent>>,
    mut piece_connection_reader: Local<ManualEventReader<PieceConnectionEvent>>,

    mut player_cursor_moved_events: ResMut<Events<PlayerCursorMovedEvent>>,
    mut player_cursor_moved_reader: Local<ManualEventReader<PlayerCursorMovedEvent>>,

    mut player_disconnected_events: ResMut<Events<PlayerDisconnectedEvent>>,
    mut player_disconnected_reader: Local<ManualEventReader<PlayerDisconnectedEvent>>,

    mut network_io: ResMut<NetworkIO>,
    mut puzzle: ResMut<Puzzle>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // forward all events generated by the client to the server

    macro_rules! forward_events {
        ($reader: ident, $events: ident) => {
            for event in $reader.iter(&$events) {
                if network_io.input.send(event.serialize()).is_err() {
                    next_state.set(AppState::ConnectionLost);
                    return;
                }
            }
        };
    }

    forward_events!(piece_moved_reader, piece_moved_events);
    forward_events!(piece_picked_up_reader, piece_picked_up_events);
    forward_events!(piece_put_down_reader, piece_put_down_events);
    forward_events!(piece_connection_reader, piece_connection_events);
    forward_events!(player_cursor_moved_reader, player_cursor_moved_events);
    forward_events!(player_disconnected_reader, player_disconnected_events);

    // receive events from the server and apply them to the local puzzle instance
    let mut new_events = Vec::new();
    while let Ok(msg) = network_io.output.try_recv() {
        let event = AnyGameEvent::deserialize(msg.as_str());
        new_events.extend(puzzle.apply_event(event));
    }

    // dispatch new events out to bevy
    for event in new_events {
        use AnyGameEvent::*;
        match event {
            PieceMoved(event) => piece_moved_events.send(event),
            PiecePickedUp(event) => piece_picked_up_events.send(event),
            PiecePutDown(event) => piece_put_down_events.send(event),
            PieceConnection(event) => piece_connection_events.send(event),
            PlayerCursorMoved(event) => player_cursor_moved_events.send(event),
            PlayerDisconnected(event) => player_disconnected_events.send(event),
        }
    }

    // consume all the events we just dispatched so we don't forward them back out next frame
    piece_moved_reader.clear(&piece_moved_events);
    piece_picked_up_reader.clear(&piece_picked_up_events);
    piece_put_down_reader.clear(&piece_put_down_events);
    piece_connection_reader.clear(&piece_connection_events);
    player_cursor_moved_reader.clear(&player_cursor_moved_events);
    player_disconnected_reader.clear(&player_disconnected_events);
}
