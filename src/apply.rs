use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::cli::ApplyArgs;

#[instrument(level = "debug", skip(args))]
pub fn run(args: ApplyArgs) -> Result<()> {
    info!(url = ?args.url, "apply stub");
    Ok(())
}
