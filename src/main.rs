use clap::Parser;
use color_eyre::eyre::Result;
use tracing::instrument;

use shortcut_catapult::{
    apply,
    cli::{Cli, Commands},
    config, daemon,
};

#[instrument(level = "trace")]
fn main() {
    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(3);
    }
}

fn run() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            err.print()?;
            std::process::exit(1);
        }
    };
    let level = cli.log_level();
    shortcut_catapult::init(level)?;
    let config_path = config::config_path(cli.config.clone())?;
    tracing::debug!(?config_path, "using config path");
    match cli.command {
        Commands::Daemon(args) => daemon::run(args, config_path)?,
        Commands::Apply(args) => apply::run(args, config_path)?,
    }
    Ok(())
}
