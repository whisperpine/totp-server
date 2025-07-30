//! Time-based One-time Password (TOTP) web server.
//!
//! This crate provides a web server for generating and validating TOTP tokens.

/// Defines constants and utilities for server configuration.
pub mod config;
/// Defines custom error types and their implementations.
pub mod error;
/// The entry point of totp_server library.
pub mod server;
/// Converts [`tower::Service`] inner errors into [`axum::response::IntoResponse`].
pub mod service;
/// Core module for Time-based One-time Password (TOTP).
pub mod totp;
/// Utility routers for fallback and health checks.
pub mod utils;

pub(crate) use config::{BIND_PORT, PKG_VERSION, RATE_LIMIT, env_var_check};
pub(crate) use service::{buffer_error_handler, timeout_error_handler};
pub(crate) use totp::{VEC_SECRET, check_current};
pub(crate) use utils::{handler_404, handler_502, health};

pub use config::CRATE_NAME;
pub use error::{Error, Result};
pub use server::start_server;
pub use totp::{InputToken, try_get_token};
