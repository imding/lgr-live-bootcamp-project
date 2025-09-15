use {crate::helpers::TestApp, serde_json::json};

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "12345"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "123456"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            // "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "123456"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 422);
}
