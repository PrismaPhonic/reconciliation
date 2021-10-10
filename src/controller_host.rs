use futures::future::join_all;
use std::error::Error;
use tokio_context::context::{Context, Handle};

use crate::controller::{Controller, ControllerExecutor};

/// ControllerHost will facilitate registering controllers by wrapping them in ControllerExecutors,
/// and beginning asynchronous execution of all ControllerExecutors, which in turn run their
/// respect controller logic.
///
/// Use `run` to run all of the registered controllers. Use `cancel_all` to cancel all controllers,
/// and wait on all controllers to finish gracefully executing.
pub struct ControllerHost<E: Error + Send + Sync + 'static> {
    executors: Vec<ControllerExecutor<E>>,
    cancel_handle: Option<Handle>,
}

impl<E> ControllerHost<E>
where
    E: Error + Send + Sync + 'static,
{
    /// Create a new ControllerHost.
    pub fn new() -> ControllerHost<E> {
        ControllerHost {
            executors: vec![],
            cancel_handle: None,
        }
    }

    /// Cancels all running executors, and blocks, waiting for them all to gracefully terminate.
    pub async fn cancel_all(&mut self) {
        if let Some(handle) = self.cancel_handle.take() {
            handle.cancel();
            join_all(self.executors.iter_mut().map(|e| e.wait())).await;
        }
    }

    /// Starts all controllers up, returning immediately. Call `cancel_all` to cancel all
    /// executors, which will also block, waiting for all executors to gracefully exit. Do not use
    /// `run` again until you have run both `cancel_all`.
    pub async fn run(&mut self) {
        let (_, mut handle) = Context::new();
        join_all(self.executors.iter_mut().map(|e| {
            let ctx = handle.spawn_ctx();
            e.start(ctx)
        }))
        .await;

        self.cancel_handle = Some(handle);
    }

    /// Adds a controller to the host. All controllers that have been added to the host will
    /// automatically have their control loops started when `run` is executed.
    pub async fn add_controller(&mut self, controller: Box<dyn Controller<Error = E>>) {
        self.executors
            .push(ControllerExecutor::new(controller).await);
    }
}

impl<E> Default for ControllerHost<E>
where
    E: Error + Sync + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
