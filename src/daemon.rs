use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::cli::DaemonArgs;

#[instrument(level = "debug", skip(args))]
pub fn run(args: DaemonArgs) -> Result<()> {
    info!(port = args.port, "daemon stub");
    Ok(())
}
