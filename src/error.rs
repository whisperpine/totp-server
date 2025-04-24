use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// A handy type alias for `Result<T, axum_demo::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Enumeration of errors that can occur in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided TOTP code does not match the expected 6-digit format.
    #[error("TOTP must be a 6-digit number")]
    TotpInvalidFormat,
    /// The provided TOTP code is invalid or expired.
    #[error("invalid TOTP")]
    TotpInvalid,
    /// An error occurred while accessing system time.
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let msg = format!("Error: {self}");
        type E = crate::Error;
        match self {
            E::TotpInvalid => (StatusCode::UNAUTHORIZED, msg).into_response(),
            E::TotpInvalidFormat => (StatusCode::BAD_REQUEST, msg).into_response(),
            E::SystemTime(_) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}
