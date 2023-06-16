use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use game::Puzzle;

use crate::{states::AppState, worker::Worker};

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), spawn_load_task)
            .add_systems(Update, handle_load_task.run_if(in_state(AppState::Loading)));
    }
}

type PuzzleLoader = Worker<(), Puzzle>;

fn spawn_load_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let loader = PuzzleLoader::spawn(thread_pool, |_, tx| async move {
        let response =
            reqwest::get("https://m.media-amazon.com/images/I/71tNdtNw70L._UF1000,1000_QL80_.jpg");
        let bytes = response.await.unwrap().bytes().await.unwrap();
        let image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::Jpeg)
            .unwrap()
            .to_rgba8();
        let puzzle = Puzzle::new(image, 36, true);
        tx.send(puzzle).unwrap();
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
