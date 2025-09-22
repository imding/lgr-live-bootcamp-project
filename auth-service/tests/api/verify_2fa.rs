use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{domain::email::Email, utils::constants::JWT_COOKIE_NAME},
    serde_json::json,
};

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await;
    let _ = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;
    let (attempt_id, code) = app
        .two_factor_store
        .get_code(&Email::parse(&email).unwrap())
        .await
        .expect("Failed to get code from two factor store");
    let response = app
        .post_verify_2fa(&json!({
            "email": email,
            "loginAttemptId": attempt_id,
            "2FACode": code
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response.cookies().find(|cookie| cookie.name() == JWT_COOKIE_NAME).expect("No auth cookie found");

    assert!(!cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "12345"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 400);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "123456"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await;
    let _ = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;
    let (attempt_id, code) = app
        .two_factor_store
        .get_code(&Email::parse(&email).unwrap())
        .await
        .expect("Failed to get code from two factor store");
    let _ = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;
    let response = app
        .post_verify_2fa(&json!({
            "email": email,
            "loginAttemptId": attempt_id,
            "2FACode": code
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await;
    let _ = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;
    let (attempt_id, code) = app
        .two_factor_store
        .get_code(&Email::parse(&email).unwrap())
        .await
        .expect("Failed to get code from two factor store");
    let _ = app
        .post_verify_2fa(&json!({
            "email": email,
            "loginAttemptId": attempt_id,
            "2FACode": code
        }))
        .await;
    let response = app
        .post_verify_2fa(&json!({
            "email": email,
            "loginAttemptId": attempt_id,
            "2FACode": code
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let response = app
        .post_verify_2fa(&json!({
            "email": "me@null.computer",
            // "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "123456"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 422);

    app.clean_up().await;
}
