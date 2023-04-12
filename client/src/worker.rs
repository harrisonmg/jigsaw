use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Resource)]
#[allow(unused)]
pub struct Worker<I, O> {
    pub input: UnboundedSender<I>,
    pub output: UnboundedReceiver<O>,
}

impl<I, O> Worker<I, O> {
    pub fn spawn<Func, Fut>(thread_pool: &AsyncComputeTaskPool, function: Func) -> Self
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
