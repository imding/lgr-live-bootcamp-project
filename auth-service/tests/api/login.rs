use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{
        ErrorResponse,
        domain::email::Email,
        routes::{RegularAuthResponse, TwoFactorAuthResponse},
        utils::constants::JWT_COOKIE_NAME,
    },
    serde_json::json,
};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";
    let response = app
        .post_signup(&json!({
            "email": email,
            "password": password,
            "requires2FA": false
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response.cookies().find(|cookie| cookie.name() == JWT_COOKIE_NAME).expect("No auth cookie found");

    assert!(!cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";
    let response = app
        .post_signup(&json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 206);

    let body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(body.message, "2FA required".to_owned());

    let email = Email::parse(&email).unwrap();
    let maybe_value = app.two_factor_store.get_code(&email).await;

    assert!(maybe_value.is_ok());

    let (attempt_id, _) = maybe_value.unwrap();

    assert_eq!(body.login_attempt_id, attempt_id);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": false
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
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": false
        }))
        .await;
    let response = app
        .post_login(&json!({
            "email": email,
            "password": "abcd7890"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let _ = app
        .post_signup(&json!({
            "email": email,
            "password": "abcd1234",
            "requires2FA": false
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
            .json::<RegularAuthResponse>()
            .await
            .expect("Could not deserialize response body to LoginResponse")
            .message
            .starts_with("Failed to deserialize the JSON body into the target type"),
    );

    app.clean_up().await;
}
