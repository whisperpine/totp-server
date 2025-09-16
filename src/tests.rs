//! Unit tests for totp-server lib.

use crate::server::app;
use axum::{http::StatusCode, routing::get};
use rstest::rstest;
use std::net::SocketAddr;
use tokio::sync::oneshot;

async fn setup_server(
    router: axum::Router,
) -> (
    SocketAddr,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<Result<(), std::io::Error>>,
) {
    // Bind to the localhost's random TCP socket.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    // Create a oneshot channel for shutdown signaling.
    let (tx, rx) = oneshot::channel::<()>();
    let server = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async { rx.await.unwrap() })
    .into_future();
    let handle = tokio::spawn(server);
    wait_until_ready(addr).await;
    (addr, tx, handle)
}

/// Wait until the server is ready.
async fn wait_until_ready(addr: SocketAddr) {
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    let client = reqwest::Client::new();
    let wait_duration = Duration::from_secs(1);
    timeout(wait_duration, async {
        loop {
            if client
                .get(format!("http://{addr}/health"))
                .send()
                .await
                .map(|res| res.status().is_success())
                .unwrap_or(false)
            {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Health check timeout");
}

#[tokio::test]
async fn test_handler_405() {
    let (addr, tx, handle) = setup_server(app()).await;
    let response = reqwest::Client::new()
        .get(format!("http://{addr}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.text().await.unwrap(), "405 Method Not Allowed");
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[tokio::test]
async fn test_handler_404() {
    let (addr, tx, handle) = setup_server(app()).await;
    let response = reqwest::Client::new()
        .get(format!("http://{addr}/somewhere/unreachable"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().await.unwrap(), "404 Not Found");
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[tokio::test]
async fn test_health() {
    let (addr, tx, handle) = setup_server(app()).await;
    let response = reqwest::Client::new()
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "200 OK");
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[rstest]
#[tokio::test]
#[case("12345")]
#[case("123")]
// 6-digits token is required.
async fn test_totp_invalid_format(#[case] false_token: &str) {
    let (addr, tx, handle) = setup_server(app()).await;
    let response = reqwest::Client::new()
        .post(format!("http://{addr}"))
        .json(&crate::InputToken::new(false_token))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[tokio::test]
async fn test_token_checker_incorrect() {
    let (addr, tx, handle) = setup_server(app()).await;
    let false_token = format!("{:0>6}", rand::random_range(0..=999999));
    let response = reqwest::Client::new()
        .post(format!("http://{addr}"))
        .json(&crate::InputToken::new(false_token))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[tokio::test]
async fn test_timeout_middleware() {
    use axum::error_handling::HandleErrorLayer;
    let router = axum::Router::new()
        .route("/health", get(crate::health))
        .route("/sleep/{seconds}", get(crate::utils::sleep_secs))
        .layer(
            tower::ServiceBuilder::new()
                .layer(HandleErrorLayer::new(crate::timeout_error_handler))
                .timeout(std::time::Duration::from_secs_f32(0.2)),
        );
    let (addr, tx, handle) = setup_server(router).await;
    let response = reqwest::Client::new()
        .get(format!("http://{addr}/sleep/5"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}

#[tokio::test]
async fn test_too_many_requests() {
    let (addr, tx, handle) = setup_server(app()).await;
    let client = reqwest::Client::new();
    // There's a health check request in `setup_server()` above,
    // which already counts 1 request, thus `- 1` is used here.
    for _ in 0..(*crate::RATE_LIMIT - 1) {
        let response = client
            .get(format!("http://{addr}/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    for _ in 0..3 {
        let response = client
            .get(format!("http://{addr}/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    tx.send(()).unwrap();
    let _ = handle.await.unwrap();
}
