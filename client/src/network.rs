use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_util::{SinkExt, StreamExt};
use game::Puzzle;
use ws_stream_wasm::{WsMessage, WsMeta};

use crate::states::AppState;
use crate::worker::Worker;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), spawn_network_io_task)
            .add_systems(
                Update,
                handle_network_io_task.run_if(in_state(AppState::Loading)),
            );
    }
}

type NetworkIO = Worker<(), String>;

fn spawn_network_io_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let io = NetworkIO::spawn(thread_pool, |_, tx| async move {
        let (_, ws_io) = WsMeta::connect("ws://127.0.0.1:3030/client", None)
            .await
            .unwrap();
        let (mut _ws_tx, mut ws_rx) = ws_io.split();
        loop {
            if let Some(WsMessage::Text(msg)) = ws_rx.next().await {
                tx.send(msg).unwrap();
            }
            // TODO
            break;
        }
    });
    commands.insert_resource(io);
}

fn handle_network_io_task(
    mut commands: Commands,
    mut io: ResMut<NetworkIO>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    while let Ok(msg) = io.output.try_recv() {
        // TODO something is going wrong here
        let puzzle = Puzzle::from(msg.as_str());
        commands.insert_resource(puzzle);
        next_state.set(AppState::Setup);
    }
}
