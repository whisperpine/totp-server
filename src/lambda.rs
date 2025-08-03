/// Starts the HTTP server for the TOTP service (AWS Lambda).
pub async fn start_server_aws_lambda() {
    tracing::info!("app version: {}", crate::PKG_VERSION);
    // Check if required env vars have been set correctly.
    let _ = crate::VEC_SECRET.clone();
    // Start the server by `lambda_http::run`, which differs from `axum::serve`.
    lambda_http::run(app_aws_lambda())
        .await
        .unwrap_or_else(|e| panic!("failed to start lambda_http server. error: {e}"));
}

pub(crate) fn app_aws_lambda() -> axum::Router {
    use crate::*;
    use axum::error_handling::HandleErrorLayer;
    use axum::routing::get;

    axum::Router::new()
        .route("/", get(handler_502).post(check_current))
        .route("/health", get(health))
        .fallback(handler_404)
        .layer(
            tower::ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_error_handler))
                .timeout(std::time::Duration::from_secs(1)),
        )
}
