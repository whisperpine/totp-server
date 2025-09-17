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
    // Check if required env vars have been set correctly.
    crate::env_var_check();
    // Print the URL and QR Code to stdout.
    crate::print_qr_code();

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

    // Configure the rate limiter.
    let governor_conf = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(*crate::RATE_LIMIT)
            .finish()
            .expect("failed to configure tower_governor"),
    );

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    // A separate background task to clean up.
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            tracing::trace!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    axum::Router::new()
        .route("/", get(handler_405).post(check_current))
        .route("/health", get(health))
        .fallback(handler_404)
        .layer(
            tower::ServiceBuilder::new()
                // Handle timeout error.
                .layer(HandleErrorLayer::new(timeout_error_handler))
                // Handle timeout.
                .timeout(Duration::from_secs(1))
                // Handle request rate limits.
                .layer(tower_governor::GovernorLayer::new(governor_conf)),
        )
}
