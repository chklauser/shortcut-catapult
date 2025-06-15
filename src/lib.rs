use color_eyre::eyre::Result;
use tracing_subscriber::{EnvFilter, prelude::*};

pub mod apply;
pub mod cli;
pub mod config;
pub mod daemon;
pub mod matching;

/// Initialize error handling and tracing.
///
/// The log level can be configured via the `CATAPULT_LOG` environment variable.
/// The default level is `warning`.
use once_cell::sync::OnceCell;

static INIT: OnceCell<()> = OnceCell::new();

pub fn init(level: Option<tracing::Level>) -> Result<()> {
    INIT.get_or_try_init(|| {
        color_eyre::install()?;
        tracing_log::LogTracer::init().ok();
        let filter = match level {
            Some(level) => EnvFilter::new(level.as_str()),
            None => {
                EnvFilter::try_from_env("CATAPULT_LOG").unwrap_or_else(|_| EnvFilter::new("warn"))
            }
        };
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .ok();
        Ok(())
    })
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_ok() {
        init(None).expect("init should not error");
    }
}
