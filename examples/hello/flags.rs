use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "reconciler",
    about = "reconciles user intent with system state"
)]
pub struct Flags {
    // TODO: Plumb this through and add logging. Probably also change this to actually be log
    // level enum.
    /// Log level: error, warn, info, debug, trace
    #[structopt(long, env = "LOG_LEVEL", default_value = "error")]
    pub log_level: String,

    // TODO: Probably make all three of these generalist rather than mysql specific.
    // Also, this contains the username and password as part of the connection string.
    // We should probably load those from a file and instead ask for sub-sets of the address and
    // compose it ourselves.
    /// The full connection string to connect to mysql, including database name
    #[structopt(long, env = "DATABASE_URL")]
    pub mysql_url: String,

    /// Reconcile controllers with this period even if no events occur
    #[structopt(long, env = "RESYNC_PERIOD_SECONDS", default_value = "5")]
    pub resync_period_seconds: u64,

    /// Provides a default retention period, by which we will clean up soft deleted data if it has
    /// been around beyond the number of days specified
    #[structopt(long, env = "RESYNC_PERIOD_SECONDS", default_value = "3")]
    pub retention_period_days: u64,
}
