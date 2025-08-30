use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{ErrorResponse, routes::LoginResponse},
    serde_json::json,
};

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": true
        }))
        .await;
    let response = app
        .post_login(&json!({
            "email": email,
            "password": "abcd123"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response.json::<ErrorResponse>().await.expect("Could not deserialize response body to ErrorResponse").error,
        "Invalid credentials".to_owned()
    )
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": true
        }))
        .await;
    let response = app
        .post_login(&json!({
            "email": email,
            "password": "abcd7890"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": true
        }))
        .await;
    let response = app
        .post_login(&json!({
            "email": email,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 422);
    assert!(
        response
            .json::<LoginResponse>()
            .await
            .expect("Could not deserialize response body to LoginResponse")
            .message
            .starts_with("Failed to deserialize the JSON body into the target type"),
    )
}
