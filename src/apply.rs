use std::io::{self, Read, Write};
use std::path::PathBuf;

use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::{cli::ApplyArgs, config::Config, matching::Matcher};

/// Log the trace using the dedicated trace category
fn log_trace(trace: &crate::matching::LogTrace) {
    tracing::info!(target: "shortcut_catapult::trace", "OK: {}", trace);
}

#[instrument(level = "debug", skip(args, config_path))]
pub fn run(args: ApplyArgs, config_path: PathBuf) -> Result<()> {
    info!(url = ?args.url, ?config_path, "apply start");

    let mut input = match args.url.as_deref() {
        Some("-") | None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
        Some(val) => val.to_string(),
    };
    input = input.trim_end_matches(&['\n', '\r'][..]).to_string();

    let cfg_str = crate::config::read(&config_path)?;
    let cfg: Config = Config::parse(&cfg_str)?;

    match cfg.matcher.apply(&input)? {
        Some((url, trace)) => {
            log_trace(&trace);
            print!("{}", url);
            io::stdout().flush()?;
            Ok(())
        }
        None => {
            std::process::exit(2);
        }
    }
}
