//! Time-based One-time Password (TOTP) web server.
//!
//! This crate provides a web server for generating and validating TOTP tokens.

// rustc
#![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(not(debug_assertions), deny(missing_docs))]
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), deny(warnings))]
// clippy
#![cfg_attr(not(debug_assertions), deny(clippy::todo))]
#![cfg_attr(
    not(any(test, debug_assertions)),
    deny(clippy::print_stdout, clippy::dbg_macro)
)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::net::SocketAddr;
    use totp_server::{BIND_PORT, PKG_VERSION};
    use tracing::info;

    init_tracing_subscriber();
    info!("app version: {}", PKG_VERSION);
    totp_server::env_var_check();

    let addr = SocketAddr::from(([0, 0, 0, 0], *BIND_PORT));
    info!("listening at http://localhost:{}", addr.port());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app()).await?;

    Ok(())
}

fn init_tracing_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", totp_server::CRATE_NAME).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn app() -> axum::Router {
    use axum::error_handling::HandleErrorLayer;
    use axum::routing::get;
    use std::time::Duration;
    use totp_server::*;
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
