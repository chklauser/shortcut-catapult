use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use reqwest::StatusCode;
use shortcut_catapult::{self, daemon};
use tracing::Level;

const EXACT_CONFIG: &str = "match:\n  exact: Hello\n  url: https://example.com?q=$1\n";

async fn spawn_server(
    config: &str,
) -> (
    tokio::task::JoinHandle<std::io::Result<()>>,
    std::net::SocketAddr,
    NamedTempFile,
) {
    shortcut_catapult::init(Some(Level::DEBUG)).ok();
    let file = NamedTempFile::new("config.yml").expect("temp file");
    file.write_str(config).expect("write config");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    let app = daemon::test_router(file.path().to_path_buf());
    let server = axum::serve(listener, app).into_future();

    let handle = tokio::spawn(server);
    (handle, addr, file)
}

#[tokio::test]
async fn redirect_returns_302() {
    let (handle, addr, _file) = spawn_server(EXACT_CONFIG).await;

    let url = format!("http://{}:{}/Hello", addr.ip(), addr.port());
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let resp = client.get(&url).send().await.expect("request");
    eprintln!("status: {}", resp.status());
    eprintln!("headers: {:?}", resp.headers());
    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(
        resp.headers().get("location").unwrap(),
        "https://example.com?q=Hello"
    );

    handle.abort();
}

#[tokio::test]
async fn not_found_returns_404() {
    let (handle, addr, _file) = spawn_server(EXACT_CONFIG).await;

    let url = format!("http://{}:{}/World", addr.ip(), addr.port());
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let resp = client.get(&url).send().await.expect("request");
    eprintln!("status2: {}", resp.status());
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert!(resp.text().await.unwrap().is_empty());

    handle.abort();
}
