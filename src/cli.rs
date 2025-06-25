use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Enable INFO logging
    #[arg(long, global = true)]
    pub info: bool,
    /// Enable DEBUG logging
    #[arg(long, global = true)]
    pub debug: bool,
    /// Path to config file
    #[arg(long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn log_level(&self) -> Option<tracing::Level> {
        if self.debug {
            Some(tracing::Level::DEBUG)
        } else if self.info {
            Some(tracing::Level::INFO)
        } else {
            None
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the HTTP daemon
    Daemon(DaemonArgs),
    /// Apply the config to a single URL
    Apply(ApplyArgs),
    /// Install systemd user service and socket
    Install(InstallArgs),
    /// Uninstall systemd user service and socket
    Uninstall(UninstallArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DaemonArgs {
    /// Port to listen on
    #[arg(long, default_value = "8081")]
    pub port: u16,
    /// Enable systemd mode (socket activation and sd_notify)
    #[arg(long)]
    pub systemd: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ApplyArgs {
    /// URL to process or '-' for stdin
    pub url: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    /// Port for the systemd socket to listen on
    #[arg(long, default_value = "8081")]
    pub port: u16,
}

#[derive(Args, Debug, Clone)]
pub struct UninstallArgs {}
