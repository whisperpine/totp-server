/// Starts the HTTP server for the TOTP service (AWS Lambda).
///
/// # Panics
///
/// It panics if fails to start the Lambda Rust runtime.
pub async fn start_server_aws_lambda() {
    tracing::info!("App version: {}.", crate::PKG_VERSION);
    // Check if required env vars have been set correctly.
    let _ = crate::VEC_SECRET.clone();
    // Print the base32-encoded secret.
    crate::print_secret_base32();
    // Start the server by `lambda_http::run`, which differs from `axum::serve`.
    lambda_http::run(app_aws_lambda())
        .await
        .unwrap_or_else(|e| panic!("Failed to start lambda_http server. Error: {e}."));
}

pub(crate) fn app_aws_lambda() -> axum::Router {
    use crate::{check_current, handler_404, handler_405, health, timeout_error_handler};
    use axum::error_handling::HandleErrorLayer;
    use axum::routing::get;

    axum::Router::new()
        .route("/", get(handler_405).post(check_current))
        .route("/health", get(health))
        .fallback(handler_404)
        .layer(
            tower::ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_error_handler))
                .timeout(std::time::Duration::from_secs(1)),
        )
}

#[cfg(test)]
mod tests {
    // It's infeasible to run super::start_server_aws_lambda() in rust tests,
    // due to the runtime environment (which can be mocked by `cargo lambda`).
    #[tokio::test]
    async fn test_app_aws_lambda() {
        use super::app_aws_lambda;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let is_timeout = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            axum::serve(listener, app_aws_lambda()),
        )
        .await
        .is_err();
        assert!(is_timeout);
    }
}
