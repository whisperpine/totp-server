#[tokio::test]
async fn test_start_server() {
    use std::time::Duration;
    use tokio::time::timeout;
    // totp_server::start_server() never stop, so it's expected to timeout.
    let is_timeout = timeout(Duration::from_millis(200), totp_server::start_server())
        .await
        .is_err();
    assert!(is_timeout);
}
