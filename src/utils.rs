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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_handler() {
        use axum::routing::get;
        let _ = axum::Router::<()>::new()
            .route("/", get(handler_502))
            .route("/health", get(health))
            .fallback(handler_404);
    }
}
