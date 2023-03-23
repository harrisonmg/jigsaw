use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use game::Puzzle;

use crate::states::AppState;

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_load_task)
            .add_system(handle_load_task.run_if(in_state(AppState::Loading)));
    }
}

#[derive(Resource)]
#[allow(unused)]
struct Worker<I, O> {
    input: UnboundedSender<I>,
    output: UnboundedReceiver<O>,
}

impl<I, O> Worker<I, O> {
    fn spawn<Func, Fut>(thread_pool: &AsyncComputeTaskPool, function: Func) -> Self
    where
        Func: FnOnce(UnboundedReceiver<I>, UnboundedSender<O>) -> Fut,
        Fut: Future<Output = ()> + 'static,
    {
        let (input_tx, input_rx) = unbounded_channel::<I>();
        let (output_tx, output_rx) = unbounded_channel::<O>();
        thread_pool
            .spawn_local(function(input_rx, output_tx))
            .detach();
        Worker {
            input: input_tx,
            output: output_rx,
        }
    }
}

type PuzzleLoader = Worker<(), Puzzle>;

fn spawn_load_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let loader = PuzzleLoader::spawn(&thread_pool, |_, tx| async move {
        let response = reqwest::get("https://m.media-amazon.com/images/W/IMAGERENDERING_521856-T1/images/I/71tNdtNw70L._UF1000,1000_QL80_.jpg");
        let bytes = response.await.unwrap().bytes().await.unwrap();
        let image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::Jpeg)
            .unwrap()
            .to_rgba8();
        tx.send(Puzzle::new(image, 9)).unwrap();
    });
    commands.insert_resource(loader);
}

fn handle_load_task(
    mut commands: Commands,
    mut loader: ResMut<PuzzleLoader>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Ok(puzzle) = loader.output.try_recv() {
        commands.insert_resource(puzzle);
        commands.remove_resource::<PuzzleLoader>();
        next_state.set(AppState::Setup);
    }
}
