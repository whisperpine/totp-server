/// Starts the HTTP server for the TOTP service.
///
/// This function:
/// - Binds the server to the address specified by `BIND_PORT`.
/// - Initializes logging with the app version and environment variable checks.
/// - Starts serving requests using the `axum::serve` framework.
///
/// # Panics
///
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

    axum::serve(
        listener,
        app().into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap_or_else(|e| panic!("failed to start axum server. error: {e}"));
}

/// Configures and returns the Axum router for the TOTP service.
///
/// # Returns
///
/// An [`axum::Router`] configured with routes and middleware for the TOTP service.
pub(crate) fn app() -> axum::Router {
    use crate::*;
    use axum::error_handling::HandleErrorLayer;
    use axum::routing::get;
    use std::time::Duration;
    use tower::ServiceBuilder;

    // Configure the rate limiter.
    let governor_conf = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(*crate::config::RATE_LIMIT)
            .finish()
            .unwrap(),
    );

    axum::Router::new()
        .route("/", get(handler_502).post(check_current))
        .route("/health", get(health))
        .fallback(handler_404)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_error_handler))
                .timeout(Duration::from_secs(1))
                .layer(tower_governor::GovernorLayer {
                    config: governor_conf,
                }),
        )
}
