use clap::Parser;
use color_eyre::eyre::Result;
use tracing::instrument;

use shortcut_catapult::{
    apply,
    cli::{Cli, Commands},
    config, daemon,
};

#[instrument(level = "trace")]
fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = cli.log_level();
    shortcut_catapult::init(level)?;
    let config_path = config::config_path(cli.config.clone())?;
    tracing::debug!(?config_path, "using config path");
    match cli.command {
        Commands::Daemon(args) => daemon::run(args)?,
        Commands::Apply(args) => apply::run(args)?,
    }
    Ok(())
}
