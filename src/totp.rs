use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use totp_rs::{Algorithm, TOTP};

const TOKEN_DIGITS: usize = 6;

/// Env var used to get raw secret of TOTP.
const RAW_SECRET: &str = "RAW_SECRET";

/// Secret which should be used to construct [`totp_rs::TOTP`].
///
/// It hasn't been encoded by base32.
///
/// # Example
///
/// ```
/// # use totp_rs::{Algorithm, TOTP};
/// # let VEC_SECRET: Vec<u8> = vec![];
/// let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, VEC_SECRET.clone());
/// ```
///
/// # Panic
///
/// - When env var [`RAW_SECRET`] hasn't been set or its bitsize is smaller than 128.
/// - When [`RAW_SECRET`] is missing in release profile cfg(not(debug_assertions)).
pub(crate) static VEC_SECRET: LazyLock<Vec<u8>> = LazyLock::new(init_vec_secret);

fn init_vec_secret() -> Vec<u8> {
    match std::env::var(RAW_SECRET) {
        Ok(value) => {
            let result = value.as_bytes().to_vec();
            let bitsize = result.len() * 8;
            if bitsize < 128 {
                panic!("the bitsize of RAW_SECRET should at least 128 (the given one is {bitsize})")
            }
            result
        }
        Err(_) => {
            #[cfg(debug_assertions)]
            fn handle_error() -> Vec<u8> {
                use rand::distr::{Alphanumeric, SampleString};
                let raw_secret = Alphanumeric.sample_string(&mut rand::rng(), 32);
                tracing::info!("using random totp secret in debug build: {}", raw_secret);
                raw_secret.as_bytes().to_vec()
            }
            #[cfg(not(debug_assertions))]
            fn handle_error() -> Vec<u8> {
                panic!("env var {RAW_SECRET} should be set.")
            }
            // Get random value in debug build while panic in release build
            handle_error()
        }
    }
}

/// Create a new instance of [`TOTP`] with given parameters.
///
/// # Panics
///
/// It panics if the `digit` or `secret` size is invalid.
/// `digit` is set by [`TOKEN_DIGITS`], thus it's unlikely to be invalid.
/// `secret` must have bitsize of at least 128 or it will panic.
fn new_totp(secret: impl Into<Vec<u8>>) -> totp_rs::TOTP {
    TOTP::new(
        Algorithm::SHA1,
        TOKEN_DIGITS,
        1,
        30,
        secret.into(),
        Some(crate::PKG_NAME.to_owned()),
        "incognito".to_owned(),
    )
    .unwrap_or_else(|e| panic!("failed creating a new instance of TOTP: {e}"))
}

/// Try get totp token with raw secret.
///
/// Param `secret` should be at least 128 bit.
///
/// # Example
///
/// ```
/// use totp_server::try_get_token;
/// let vec = "999a999a999a999a".as_bytes();
/// assert!(vec.len() >= 16);
/// let token = try_get_token(&vec).unwrap();
/// ```
pub fn try_get_token(secret: &[u8]) -> crate::Result<String> {
    let totp = new_totp(secret);
    let token = totp.generate_current()?;
    Ok(token)
}

/// The 6-digits token that users input.
#[derive(Debug, Serialize, Deserialize)]
pub struct InputToken {
    token: String,
}

impl InputToken {
    /// Create a new [`InputToken`].
    pub fn new(value: impl Into<String>) -> Self {
        InputToken {
            token: value.into(),
        }
    }
}

/// Check if the given token is valid.
pub(crate) async fn check_current(Json(input_token): Json<InputToken>) -> crate::Result<()> {
    tracing::debug!(?input_token);
    let token = input_token.token;
    if token.len() != TOKEN_DIGITS || token.parse::<u32>().is_err() {
        return Err(crate::Error::TotpInvalidFormat);
    }
    let totp = new_totp(VEC_SECRET.clone());
    match totp.check_current(&token)? {
        false => Err(crate::Error::TotpInvalid),
        true => {
            tracing::debug!("correct TOTP: {token}");
            Ok(())
        }
    }
}

/// Print the base32-endcode secret by [`tracing::info!()`].
pub(crate) fn print_secret_base32() {
    let totp = new_totp(VEC_SECRET.clone());
    let secret_base32 = totp.get_secret_base32();
    tracing::info!(%secret_base32);
}

/// Print the URL and QR Code to stdout.
///
/// # Panics
///
/// Panics if the QR code cannot be constructed (e.g. when the data is too long).
pub(crate) fn print_qr_code() {
    use qrcode::render::unicode;

    let totp = new_totp(VEC_SECRET.clone());
    let url = totp.get_url();
    tracing::info!(%url);

    let code = qrcode::QrCode::new(url)
        .unwrap_or_else(|e| panic!("failed to convert the URL to QR code: {e}"));
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("\n{image}\n");
}

#[cfg(test)]
mod tests {
    #![expect(unsafe_code)]

    use super::*;
    use rstest::rstest;

    /// Get the current token.
    async fn get_token() -> crate::Result<Json<InputToken>> {
        let token = try_get_token(&VEC_SECRET)?;
        let my_token = InputToken::new(token);
        Ok(Json(my_token))
    }

    #[tokio::test]
    async fn test_token_checker_correct() {
        let my_token = get_token().await.unwrap();
        check_current(my_token).await.unwrap();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_vec_secret_default_debug() {
        assert!(std::env::var(RAW_SECRET).is_err());
        assert_eq!(VEC_SECRET.len(), 32);
    }

    #[test]
    #[should_panic]
    #[cfg(not(debug_assertions))]
    fn test_vec_secret_default_not_debug() {
        assert!(std::env::var(RAW_SECRET).is_err());
        let _ = *VEC_SECRET;
    }

    #[rstest]
    #[case("H4bY!9MP8s5a#Cm4")]
    #[case("^mzshbK&T6ng5hSNc6Lq$i")]
    #[case("jG5t*qUztkV9!FA9YxDo4j7d%nLGiD!^")]
    fn test_vec_secret_default_var(#[case] raw_secret: &str) {
        unsafe { std::env::set_var(RAW_SECRET, raw_secret) }
        assert!(VEC_SECRET.len() >= 16);
    }

    #[rstest]
    #[case("")]
    #[case("ng5NcL$i")]
    #[case("oUPH&eip*wC")]
    #[should_panic]
    fn test_vec_secret_default_var_panic(#[case] raw_secret: &str) {
        unsafe { std::env::set_var(RAW_SECRET, raw_secret) }
        let _ = *VEC_SECRET;
    }

    #[test]
    fn test_print_secret_base32() {
        print_secret_base32();
    }

    #[test]
    fn test_print_qr_code() {
        print_qr_code();
    }
}
