use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_util::{SinkExt, StreamExt};
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
        let (_, ws_io) = WsMeta::connect("ws://127.0.0.1:3030/echo", None)
            .await
            .unwrap();
        let (mut ws_tx, mut ws_rx) = ws_io.split();
        ws_tx.send(WsMessage::Text("yo".into())).await.unwrap();
        if let Some(WsMessage::Text(resp)) = ws_rx.next().await {
            tx.send(resp).unwrap();
        }
    });
    commands.insert_resource(io);
}

fn handle_network_io_task(
    mut commands: Commands,
    mut io: ResMut<NetworkIO>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Ok(resp) = io.output.try_recv() {
        info!("{resp:?}");
    }

    //if let Ok(puzzle) = io.output.try_recv() {
    //    commands.insert_resource(puzzle);
    //    commands.remove_resource::<PuzzleNetwork>();
    //    next_state.set(AppState::Setup);
    //}
}
