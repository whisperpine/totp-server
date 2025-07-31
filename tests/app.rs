//! Integration tests for totp-server application.

mod common;

use anyhow::Result;
use reqwest::StatusCode;

/// Provide a correct token.
#[tokio::test]
async fn test_totp_input_token() {
    let (mut _child, token, port) = common::setup().await;

    let res = reqwest::Client::new()
        .post(format!("http://localhost:{port}"))
        .json(&totp_server::InputToken::new(&token))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let false_token = common::get_random_6_digits();
    assert_ne!(false_token, token);
    let res = reqwest::Client::new()
        .post(format!("http://localhost:{port}"))
        .json(&totp_server::InputToken::new(false_token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

/// Provide a token in an invalid format.
#[tokio::test]
async fn test_totp_invalid_format() {
    let (mut _child, _, port) = common::setup().await;
    // 6-digits token is required, while 5-digits token is provided here.
    let false_token = "12345";

    let res = reqwest::Client::new()
        .post(format!("http://localhost:{port}"))
        .json(&totp_server::InputToken::new(false_token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "just a demo which isn't relevant to this project"]
async fn process_test() -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    let child = Command::new("sh")
        .args(["-c", "ls"])
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("output:\n{stdout}");
    Ok(())
}
