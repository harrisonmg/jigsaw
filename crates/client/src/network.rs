use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_util::{SinkExt, StreamExt};
use game::{GameEvent, PieceMovedEvent, Puzzle};
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
    let io = NetworkIO::spawn(thread_pool, |_, tx| async move {
        let (_, ws_io) = WsMeta::connect("ws://127.0.0.1:3030/client", None)
            .await
            .unwrap();
        let (mut _ws_tx, mut ws_rx) = ws_io.split();
        while let Some(WsMessage::Text(msg)) = ws_rx.next().await {
            tx.send(msg).unwrap();
        }
    });
    commands.insert_resource(io);
}

fn load_puzzle(
    mut commands: Commands,
    mut network_io: ResMut<NetworkIO>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    while let Ok(msg) = network_io.output.try_recv() {
        let puzzle = Puzzle::from(msg.as_str());
        commands.insert_resource(puzzle);
        next_state.set(AppState::Setup);
    }
}

fn event_io(
    mut piece_moved_events: ResMut<Events<PieceMovedEvent>>,
    mut piece_moved_event_reader: Local<ManualEventReader<PieceMovedEvent>>,
    mut network_io: ResMut<NetworkIO>,
) {
    for event in piece_moved_event_reader.iter(&piece_moved_events) {
        network_io.input.send(GameEvent::from(event))
    }

    while let Ok(msg) = network_io.output.try_recv() {
        info!(msg);
    }

    // consume all the PieceMovedEvents we just sent so we don't process them next frame
    piece_moved_event_reader.clear(&piece_moved_events);
}
