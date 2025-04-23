//! Time-based One-time Password (TOTP) web server.

pub mod config;
pub mod error;
/// Convert [`tower::Service`] inner error [`IntoResponse`]
pub mod service;
/// Time-based One-time Password (TOTP)
pub mod totp;
pub mod utils;

pub use config::{BIND_PORT, CRATE_NAME, PKG_VERSION, RATE_LIMIT, env_var_check};
pub use error::{Error, Result};
pub use service::{buffer_error_handler, timeout_error_handler};
pub use totp::{InputToken, check_current, try_get_token};
pub use utils::{handler_404, handler_502};
