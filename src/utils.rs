use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Routing fallback.
pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}

/// Routing fallback.
pub async fn handler_502() -> impl IntoResponse {
    (StatusCode::BAD_GATEWAY, "Not a web service")
}

/// Health check.
pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
