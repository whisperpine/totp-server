use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower::BoxError;

/// Handle errors raised by [`tower::ServiceBuilder::timeout`].
pub(crate) async fn timeout_error_handler(err: BoxError) -> impl IntoResponse {
    use tower::timeout::error::Elapsed;
    if err.is::<Elapsed>() {
        let err_msg = "request timed out".to_owned();
        tracing::error!(err_msg);
        (StatusCode::REQUEST_TIMEOUT, err_msg)
    } else {
        let err_msg = format!("internal server error: {err}");
        tracing::error!(err_msg);
        (StatusCode::INTERNAL_SERVER_ERROR, err_msg)
    }
}
