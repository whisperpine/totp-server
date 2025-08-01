use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Routing fallback.
pub(crate) async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}

/// Routing fallback.
pub(crate) async fn handler_502() -> impl IntoResponse {
    (StatusCode::BAD_GATEWAY, "Not a web service")
}

/// Health check.
pub(crate) async fn health() -> impl IntoResponse {
    StatusCode::OK
}

/// Sleep for given seconds.
///
/// This should be used to test the [`tokio::timeout::TimeoutLayer`] middleware.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) async fn sleep_secs(Path(seconds): Path<u64>) -> impl IntoResponse {
    use std::time::Duration;
    use tokio::time::sleep;
    sleep(Duration::from_secs(seconds)).await;
    StatusCode::OK
}
