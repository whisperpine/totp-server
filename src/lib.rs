//! Time-based One-time Password (TOTP) web server.
//!
//! This crate provides a web server for generating and validating TOTP tokens.

/// Defines constants and utilities for server configuration.
pub mod config;
/// Defines custom error types and their implementations.
pub mod error;
/// Converts [`tower::Service`] inner errors into [`axum::response::IntoResponse`].
pub mod service;
/// Core module for Time-based One-time Password (TOTP).
pub mod totp;
/// Utility routers for fallback and health checks.
pub mod utils;

pub use config::{BIND_PORT, CRATE_NAME, PKG_VERSION, RATE_LIMIT, env_var_check};
pub use error::{Error, Result};
pub use service::{buffer_error_handler, timeout_error_handler};
pub use totp::{InputToken, check_current, try_get_token};
pub use utils::{handler_404, handler_502, health};
