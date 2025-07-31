#![cfg_attr(test, allow(unused))]

use tokio::process::Child;

/// Executable file path (e.g. `./target/debug/totp-server`).
const EXECUTABLE_PATH: &str = env!(concat!("CARGO_BIN_EXE_", "totp-server"));

fn get_available_port() -> u16 {
    use std::net::TcpListener;

    TcpListener::bind("0.0.0.0:0")
        .unwrap_or_else(|e| panic!(r#"failed to bind to "0.0.0.0:0": {e}"#))
        .local_addr()
        .unwrap_or_else(|e| panic!("failed to get local addr: {e}"))
        .port()
}

/// Spawn a child process to run totp-server.
/// The process will be killed on drop.
#[must_use]
fn spawn_totp_process(raw_secret: &str, port: u16) -> Child {
    use std::process::Stdio;
    use tokio::process::Command;
    Command::new(EXECUTABLE_PATH)
        .env("RAW_SECRET", raw_secret)
        .env("TCP_BIND_PORT", port.to_string())
        .stdout(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn child process")
}

pub(crate) fn get_random_secret() -> String {
    use rand::distr::{Alphanumeric, SampleString};
    Alphanumeric.sample_string(&mut rand::rng(), 32)
}

/// Wait until totp-server is ready.
async fn wait_until_ready(port: u16) {
    use reqwest::Client;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    let client = Client::new();
    let wait_duration = Duration::from_secs(1);

    timeout(wait_duration, async {
        loop {
            if client
                .get(format!("http://localhost:{port}/health"))
                .send()
                .await
                .map(|res| res.status().is_success())
                .unwrap_or(false)
            {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Health check timeout");
}

/// Sets up a test environment for the TOTP server.
///
/// This function:
/// - Generates a random secret for the TOTP server.
/// - Spawns a child process running the TOTP server with the generated secret.
/// - Waits until the server is ready to accept requests.
///
/// # Returns
/// A tuple containing:
/// - The child process handle (`Child`).
/// - The generated TOTP token (`String`).
/// - The port number the server is bound to (`u16`).
///
/// # Panics
/// It panics when failed to generate a token. In this case,
/// it's probably because system time the server is not correct.
#[must_use]
pub(crate) async fn setup() -> (Child, String, u16) {
    let raw_secret = get_random_secret();
    let port = get_available_port();
    let child = spawn_totp_process(&raw_secret, port);

    let token = totp_server::try_get_token(raw_secret.as_bytes()).unwrap();
    wait_until_ready(port).await;

    (child, token, port)
}
