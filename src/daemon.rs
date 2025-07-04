use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{Router, routing::get};
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::{cli::DaemonArgs, config::Config, matching::Matcher, systemd};

#[derive(Clone)]
struct AppState {
    config_path: PathBuf,
}

#[instrument(level = "info", skip(state))]
async fn handler(State(state): State<AppState>, Path(path): Path<String>) -> Response {
    let input = path.trim_start_matches('/');

    // We re-read the configuration on every request for now
    let cfg_str = match crate::config::read_async(&state.config_path).await {
        Ok(s) => s,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
        }
    };
    let cfg: Config = match Config::parse(&cfg_str) {
        Ok(cfg) => cfg,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
        }
    };

    match cfg.matcher.apply(input) {
        Ok(Some(url)) => (StatusCode::FOUND, [(header::LOCATION, url)]).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

fn router(config_path: PathBuf) -> Router {
    async fn root_handler(State(state): State<AppState>) -> Response {
        handler(State(state), Path(String::new())).await
    }

    Router::new()
        .route("/", get(root_handler))
        // `/{*path}` captures the entire rest of the request path, including
        // multiple segments. This is required to support URLs like
        // `/foo/bar` which would otherwise only match the first segment.
        .route("/{*path}", get(handler))
        .with_state(AppState { config_path })
}

pub async fn serve_http(args: DaemonArgs, config_path: PathBuf) -> Result<()> {
    info!(port = args.port, systemd = args.systemd, "daemon starting");

    let listener = if args.systemd {
        // Use socket activation from systemd
        let fds = systemd::get_systemd_listeners()?;
        if fds.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "No socket file descriptors from systemd"
            ));
        }

        // Use the first file descriptor
        let fd = fds[0];

        // SAFETY: This unsafe block is required for systemd socket activation.
        // systemd passes valid listening socket file descriptors to the service
        // via the LISTEN_FDS environment variable. We have already:
        // 1. Verified we're running under systemd (LISTEN_PID matches our process)
        // 2. Confirmed that systemd provided at least one file descriptor
        // 3. systemd guarantees these are valid listening sockets
        //
        // Taking ownership of the fd with from_raw_fd is the standard pattern
        // for systemd socket activation in Rust. There is no safe alternative
        // for working with raw file descriptors from the system.
        unsafe {
            // Convert raw fd to TcpListener
            use std::os::unix::io::FromRawFd;
            let std_listener = std::net::TcpListener::from_raw_fd(fd);
            std_listener.set_nonblocking(true)?;
            tokio::net::TcpListener::from_std(std_listener)?
        }
    } else {
        // Regular TCP listener
        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        tokio::net::TcpListener::bind(addr).await?
    };

    let app = router(config_path);

    // Notify systemd that we're ready (only in systemd mode)
    if args.systemd {
        systemd::notify_ready()?;
    }

    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(level = "debug", skip(args, config_path))]
pub fn run(args: DaemonArgs, config_path: PathBuf) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve_http(args, config_path))
}

pub fn test_router(config_path: PathBuf) -> Router {
    router(config_path)
}
