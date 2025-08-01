#[tokio::test]
async fn test_start_server() {
    // totp_server::start_server() never stop, so it's expected to timeout.
    let is_timeout = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        totp_server::start_server(),
    )
    .await
    .is_err();
    assert!(is_timeout);
}
