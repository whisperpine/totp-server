use std::sync::LazyLock;

/// Program version.
pub(crate) const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name.
pub const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

/// Package name.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Env var which is used to set [`RATE_LIMIT`]
const REQUEST_RATE_LIMIT: &str = "REQUEST_RATE_LIMIT";

/// Request rate limit in every 30 seconds.
///
/// If env var `REQUEST_RATE_LIMIT` hans't been set, the default value 25 will be set.
pub(crate) static RATE_LIMIT: LazyLock<u32> = LazyLock::new(init_rate_limit);

fn init_rate_limit() -> u32 {
    match std::env::var(REQUEST_RATE_LIMIT) {
        Ok(value) => value
            .parse::<u32>()
            .inspect(|&v| {
                if v == 0 {
                    panic!("{REQUEST_RATE_LIMIT} must not be 0");
                }
            })
            .unwrap_or_else(|_| panic!("{REQUEST_RATE_LIMIT} must be an unsigned integer!")),
        Err(_) => {
            let default_value: u32 = 25;
            tracing::info!(
                "env var {} hasn't been set. using default value: {}",
                REQUEST_RATE_LIMIT,
                default_value
            );
            default_value
        }
    }
}

/// Env var which is used to set [`BIND_PORT`]
const TCP_BIND_PORT: &str = "TCP_BIND_PORT";

/// TCP port to bind.
///
/// If env var `TCP_BIND_PORT` hans't been set, the default value 7392 will be set.
///
/// # Panics
///
/// Panics when TCP_BIND_PORT isn't a non-zero u16.
pub(crate) static BIND_PORT: LazyLock<u16> = LazyLock::new(init_bind_port);

fn init_bind_port() -> u16 {
    match std::env::var(TCP_BIND_PORT) {
        Ok(value) => value
            .parse::<u16>()
            .inspect(|value| {
                if *value == 0 {
                    panic!("TCP_LISTENER_PORT should be non-zero")
                }
            })
            .expect("TCP_LISTENER_PORT cannot be parsed to u16"),
        Err(_) => {
            let default_value: u16 = 9000;
            tracing::info!(
                "env var {} hasn't been set. using default value: {}",
                TCP_BIND_PORT,
                default_value
            );
            default_value
        }
    }
}

/// Check if required env vars have been set correctly.
///
/// Required env vars include: `RAW_SECRET`.
/// Optional env vars include: `REQUEST_RATE_LIMIT`, `TCP_BIND_PORT`.
///
/// # Panics
/// It panics when any one of the required env var hasn't been set.
/// It also panics when any parsing fails.
pub(crate) fn env_var_check() {
    let _ = crate::VEC_SECRET.clone();
    let _ = *RATE_LIMIT;
    let _ = *BIND_PORT;
}

#[cfg(test)]
mod tests {
    #![expect(unsafe_code)]

    use super::*;
    use rstest::rstest;

    #[test]
    fn test_rate_limit_default() {
        assert!(std::env::var(REQUEST_RATE_LIMIT).is_err());
        assert_eq!(*RATE_LIMIT, 25);
    }

    #[rstest]
    #[case("333")]
    #[case("22")]
    fn test_rate_limit_var(#[case] value: &str) {
        unsafe { std::env::set_var(REQUEST_RATE_LIMIT, value) }
        assert_eq!(*RATE_LIMIT, value.parse::<u32>().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_rate_limit_var_panic() {
        unsafe { std::env::set_var(REQUEST_RATE_LIMIT, "0") }
        let _ = *RATE_LIMIT;
    }

    #[test]
    fn test_bind_port_default() {
        assert!(std::env::var(TCP_BIND_PORT).is_err());
        assert_eq!(*BIND_PORT, 9000);
    }

    #[rstest]
    #[case("55555")]
    #[case("4444")]
    #[case("333")]
    #[case("22")]
    fn test_bind_port_var(#[case] port: &str) {
        unsafe { std::env::set_var(TCP_BIND_PORT, port) }
        assert_eq!(*BIND_PORT, port.parse::<u16>().unwrap());
    }

    #[rstest]
    #[case("abc")]
    #[case("-5")]
    #[case("0")]
    #[case("65536")]
    #[case("65537")]
    #[should_panic]
    fn test_bind_port_var_panic(#[case] value: &str) {
        unsafe { std::env::set_var(TCP_BIND_PORT, value) }
        let _ = *BIND_PORT;
    }

    #[test]
    fn test_cargo_name_pkg_name() {
        assert_eq!(CRATE_NAME, "totp_server"); // underscore
        assert_eq!(PKG_NAME, "totp-server"); // hyphen
    }
}
