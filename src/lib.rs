//! Time-based One-time Password (TOTP) web server.
//!
//! This crate provides a web server for generating and validating TOTP tokens.

// rustc
// #![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(not(debug_assertions), deny(missing_docs))]
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), deny(warnings))]
// clippy
#![cfg_attr(not(debug_assertions), deny(clippy::todo))]
#![cfg_attr(
    not(any(test, debug_assertions)),
    deny(clippy::print_stdout, clippy::dbg_macro)
)]

/// Defines constants and utilities for server configuration.
mod config;
/// Defines custom error types and their implementations.
mod error;
/// AWS Lambda
mod lambda;
/// The entry point of totp_server library.
mod server;
/// Converts [`tower::Service`] inner errors into [`axum::response::IntoResponse`].
mod service;
/// Core module for Time-based One-time Password (TOTP).
mod totp;
/// Utility routers for fallback and health checks.
mod utils;

#[cfg(test)]
mod tests;

pub(crate) use config::{BIND_PORT, PKG_VERSION, RATE_LIMIT, env_var_check};
pub(crate) use service::timeout_error_handler;
pub(crate) use totp::{VEC_SECRET, check_current, print_qr_code, print_secret_base32};
pub(crate) use utils::{handler_404, handler_502, health};

pub use config::{CRATE_NAME, PKG_NAME};
pub use error::{Error, Result};
pub use lambda::start_server_aws_lambda;
pub use server::start_server;
pub use totp::{InputToken, try_get_token};
