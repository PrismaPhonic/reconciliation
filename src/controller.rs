//! Creates control loops around the provided business logic.

use std::error::Error;
use std::sync::Arc;
use tracing::{error, info};

use async_trait::async_trait;
use tokio::{sync::Mutex, task::JoinHandle, time::interval};
use tokio_context::context::Context;

/// Defines the required methods that must be implemented to specify the behavior of a given
/// Controller instance.
#[async_trait]
pub trait Controller: Send + Sync {
    /// Provide an error type that this controller should return. If you would like to run it
    /// alongside other controllers within a single ControllerHost, then the Error type for each
    /// Controller must be the same.
    type Error: Error + 'static + Sync + Send;

    /// Provide initial setup for the given Controller if necessary, otherwise simply return `Ok`.
    async fn initialize(&mut self) -> Result<(), Self::Error>;

    /// Provide the necessary reconciliation logic for this controller. This generally requires
    /// fetching all of the specs that the controller is responsible for reconciling, doing some
    /// necessary work, and then updating the relevant status for the spec the controller is in
    /// charge of.
    async fn reconcile(&mut self) -> Result<(), Self::Error>;

    /// Provide the necessary logic to handle cleaning up soft deleted specs that have stayed
    /// around passed an acceptable retention period, as defined by the controller.
    async fn cleanup(&mut self) -> Result<(), Self::Error>;

    /// Retrieve the resync period for this controller. The resync period is how often this
    /// controller will engage its control loop (reconciliation and deletion) even if it has
    /// received no triggering events.
    async fn resync_period(&self) -> std::time::Duration;
}

/// A wrapper type that ensures we can send a given controller between tasks safely.
struct AsyncSafeController<E: Error + Sync + Send + 'static>(
    Arc<Mutex<Box<dyn Controller<Error = E>>>>,
);

impl<E: Error + Sync + Send + 'static> Clone for AsyncSafeController<E> {
    fn clone(&self) -> Self {
        AsyncSafeController(self.0.clone())
    }
}

impl<E> From<Box<dyn Controller<Error = E>>> for AsyncSafeController<E>
where
    E: Error + Sync + Send + 'static,
{
    fn from(controller: Box<dyn Controller<Error = E>>) -> Self {
        AsyncSafeController(Arc::new(Mutex::new(controller)))
    }
}

#[async_trait]
impl<E> Controller for AsyncSafeController<E>
where
    E: Error + Sync + Send + 'static,
{
    type Error = E;

    async fn initialize(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.initialize().await
    }

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.reconcile().await
    }

    async fn cleanup(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.cleanup().await
    }

    // TODO: Had to add Sync to the Controller constraints specifically so this layer could be
    // verified that sending Duration was safe. Try to think of a better solution. Seems silly to
    // add Sync just for this.
    async fn resync_period(&self) -> std::time::Duration {
        self.0.lock().await.resync_period().await
    }
}

pub struct ControllerExecutor<E: Error + Sync + Send + 'static> {
    /// Holds the controller we will facilitate executing a control loop around.
    controller: AsyncSafeController<E>,
    /// Holds the resync period that was retrieved from calling `resync_period` on the given
    /// controller we facilitate execution of.
    resync_period: std::time::Duration,
    /// Closed when the control loop has ended.
    done_chan: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl<E> ControllerExecutor<E>
where
    E: Error + Sync + Send + 'static,
{
    /// Create a new ControllerExecutor. Essentially the same as a From impl. The reason this is a
    /// new constructor is because you can't have an async From impl.
    pub async fn new(controller: Box<dyn Controller<Error = E>>) -> ControllerExecutor<E> {
        let resync_period = controller.resync_period().await;
        ControllerExecutor {
            controller: AsyncSafeController::from(controller),
            resync_period,
            done_chan: None,
        }
    }

    /// Waits for the control loop to gracefully exit, blocking until it has.
    pub async fn wait(&mut self) {
        if let Some(rx) = &mut self.done_chan {
            rx.await.unwrap();
        }
    }

    /// Begin execution of the concrete control loop that facilitates executing the underlying
    /// logic of the controller we are an executor for.
    pub async fn start(&mut self, mut ctx: Context) -> JoinHandle<()> {
        let mut interval = interval(self.resync_period);
        let mut controller = self.controller.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.done_chan = Some(rx);

        tokio::task::spawn(async move {
            loop {
                if controller.initialize().await.is_ok() {
                    break;
                }

                // Wait for the next tick, or until we're told to quit.
                tokio::select! {
                    _ = interval.tick() => {
                        continue;
                    },
                    _ = ctx.done() => {
                        info!("Aborting controller initialization");
                        return;
                    }
                }
            }

            info!("Starting control loop");
            loop {
                if let Err(e) = controller.reconcile().await {
                    error!("controller reconcile failed: {}", e);
                }

                if let Err(e) = controller.cleanup().await {
                    error!("controller cleanup failed: {}", e);
                }

                tokio::select! {
                    _ = interval.tick() => {
                        continue;
                    },
                    _ = ctx.done() => {
                        break;
                    }
                }
            }

            info!("Control loop terminated");

            tx.send(()).unwrap();
        })
    }
}
