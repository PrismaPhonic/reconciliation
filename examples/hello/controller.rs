use async_trait::async_trait;
use chrono::Utc;
use reconciliation::controller::Controller;

use crate::{data_access::Hellos, error::Error, models::HelloStatus};

/// The controller that will reconcile the hello table and it's related hello_status table.
pub struct HelloController {
    hellos: Hellos,
    resync_period: std::time::Duration,
    retention_period: chrono::Duration,
}

impl HelloController {
    pub fn new(
        hellos: Hellos,
        resync_period: std::time::Duration,
        retention_period: chrono::Duration,
    ) -> HelloController {
        HelloController {
            hellos,
            resync_period,
            retention_period,
        }
    }
}

#[async_trait]
impl Controller for HelloController {
    type Error = Error;

    // Nothing to do here.
    async fn initialize(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn reconcile(&mut self) -> Result<(), Error> {
        // Fetch all hellos.
        for hello in &mut self.hellos.all().await? {
            let message = format!("Hello, {}!", hello.name);
            if let Some(status) = &mut hello.status {
                if status.message == message {
                    // We don't want to issue unnecessary updates. In a real reconcilier we would
                    // likely compute much deeper equality checking.
                    continue;
                } else {
                    status.message = message;
                    status.updated_at = Utc::now();
                };
            } else {
                let status = HelloStatus {
                    hello_id: hello.id,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                    message,
                };
                hello.status = Some(status);
            }

            // This is a very simple example, so we insert one at a time. In a real
            // reconciler we should be batch inserting with prepare and execute.
            self.hellos.upsert(&hello).await?;
        }

        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Error> {
        for hello in &mut self.hellos.all_deleted(self.retention_period).await? {
            // This is a very simple example, so we delete one at a time. In a real
            // reconciler we should be batch deleting in this step.
            self.hellos.remove(&hello.id).await?;
        }

        Ok(())
    }

    async fn resync_period(&self) -> std::time::Duration {
        self.resync_period
    }
}
