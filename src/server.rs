/// Starts the HTTP server.
///
/// ## Panics
/// This function will panic if:
/// - The server fails to bind to the specified [`SocketAddr`](std::net::SocketAddr).
/// - The server fails to start serving requests ([`axum::serve()`]).
pub async fn start_server() {
    use std::net::SocketAddr;

    tracing::info!("app version: {}", crate::PKG_VERSION);
    crate::env_var_check();

    let addr = SocketAddr::from(([0, 0, 0, 0], *crate::BIND_PORT));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind SocketAddr: {addr}. error: {e}"));
    tracing::info!("listening at http://localhost:{}", addr.port());

    axum::serve(listener, app())
        .await
        .unwrap_or_else(|e| panic!("failed to start axum server. error: {e}"));
}

fn app() -> axum::Router {
    use crate::*;
    use axum::error_handling::HandleErrorLayer;
    use axum::routing::get;
    use std::time::Duration;
    use tower::ServiceBuilder;

    axum::Router::new()
        .route("/", get(handler_502).post(check_current))
        .route("/health", get(health))
        .fallback(handler_404)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_error_handler))
                .timeout(Duration::from_secs(1))
                .layer(HandleErrorLayer::new(buffer_error_handler))
                .buffer(1)
                .rate_limit(*RATE_LIMIT, Duration::from_secs(30)),
        )
}
