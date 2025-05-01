use std::sync::LazyLock;

/// Program version.
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name.
pub const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

/// Env var which is used to set [`RATE_LIMIT`]
const REQUEST_RATE_LIMIT: &str = "REQUEST_RATE_LIMIT";

/// Request rate limit in every 30 seconds.
///
/// If env var `REQUEST_RATE_LIMIT` hans't been set, the default value 25 will be set.
pub static RATE_LIMIT: LazyLock<u64> = LazyLock::new(|| match std::env::var(REQUEST_RATE_LIMIT) {
    Ok(value) => value
        .parse::<u64>()
        .expect("REQUEST_RATE_LIMIT should be an unsigned integer!"),
    Err(_) => {
        let default_value: u64 = 25;
        tracing::info!(
            "env var {} hasn't been set. using default value: {}",
            REQUEST_RATE_LIMIT,
            default_value
        );
        default_value
    }
});

/// Env var which is used to set [`BIND_PORT`]
const TPC_BIND_PORT: &str = "TPC_BIND_PORT";

/// TCP port to bind.
///
/// If env var `TPC_BIND_PORT` hans't been set, the default value 7392 will be set.
pub static BIND_PORT: LazyLock<u16> = LazyLock::new(|| match std::env::var(TPC_BIND_PORT) {
    Ok(value) => value
        .parse::<u16>()
        .expect("TCP_LISTENER_PORT cannot be parsed to u16"),
    Err(_) => {
        let default_value: u16 = 7392;
        tracing::info!(
            "env var {} hasn't been set. using default value: {}",
            TPC_BIND_PORT,
            default_value
        );
        default_value
    }
});

/// Check if required env vars have been set correctly.
///
/// Required env vars include: `RAW_SECRET`.
/// Optional env vars include: `REQUEST_RATE_LIMIT`, `TPC_BIND_PORT`.
///
/// # Panics
/// It panics when any one of the required env var hasn't been set.
/// It also panics when any parsing fails.
pub fn env_var_check() {
    let _ = crate::totp::VEC_SECRET.clone();
    let _ = *RATE_LIMIT;
    let _ = *BIND_PORT;
}
