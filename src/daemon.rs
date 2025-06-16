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

use crate::{cli::DaemonArgs, config::Config, matching::Matcher};

#[derive(Clone)]
struct AppState {
    config_path: PathBuf,
}

#[instrument(level = "info", skip(state))]
async fn handler(State(state): State<AppState>, Path(path): Path<String>) -> Response {
    let input = path.trim_start_matches('/');

    // We re-read the configuration on every request for now
    let cfg_str = match tokio::fs::read_to_string(&state.config_path).await {
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
        .route("/{path}", get(handler))
        .with_state(AppState { config_path })
}

pub async fn serve_http(args: DaemonArgs, config_path: PathBuf) -> Result<()> {
    info!(port = args.port, "daemon starting");
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let app = router(config_path);
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
