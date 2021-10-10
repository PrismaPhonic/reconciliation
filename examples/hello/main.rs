//! # Hello
//!
//! This example demonstrates a simple hello world example involving the controller paradigm
//! established by the reconciler package. This binary involves setting up a controller-host that
//! only runs the hello controller.
//!
//! Typically, many controllers would be bundled up in a given controller-host.

use chrono::Duration;
use reconciliation::controller_host::ControllerHost;
use sqlx::MySqlPool;
use structopt::StructOpt;
use tokio::signal::unix::{signal, SignalKind};

mod controller;
mod data_access;
mod error;
mod flags;
mod models;

use controller::HelloController;
use data_access::Hellos;
use error::Error;
use flags::Flags;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let flags = Flags::from_args();
    let pool = MySqlPool::connect(&flags.mysql_url).await?;
    let hellos = Hellos::new(pool);

    let mut host = ControllerHost::new();
    host.add_controller(Box::new(HelloController::new(
        hellos,
        // We set these both short to make the demo app very responsive.
        Duration::seconds(5).to_std().unwrap(),
        Duration::seconds(10),
    )))
    .await;

    host.run().await;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = sigterm.recv() => {
            eprintln!("received sigterm");
        },
        _ = sigint.recv() => {
            eprintln!("received sigint");
        },
    }

    host.cancel_all().await;

    Ok(())
}
