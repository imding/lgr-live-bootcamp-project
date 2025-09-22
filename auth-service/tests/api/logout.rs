use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{ErrorResponse, utils::constants::JWT_COOKIE_NAME},
    reqwest::Url,
    serde_json::json,
};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";

    app.post_signup(&json!({
        "email": email,
        "password": password,
        "requires2FA": false
    }))
    .await;

    let response = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await;
    let token = response.cookies().find(|cookie| cookie.name() == JWT_COOKIE_NAME).expect("No auth cookie found");

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);
    assert!(app.banned_token_store.check(token.value()).await);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = "abcd1234";

    app.post_signup(&json!({
        "email": email,
        "password": password,
        "requires2FA": false
    }))
    .await;

    app.post_login(&json!({
        "email": email,
        "password": password,
    }))
    .await;

    app.post_logout().await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response.json::<ErrorResponse>().await.expect("Could not deserialize response body to ErrorResponse").error,
        "Missing token".to_string()
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response.json::<ErrorResponse>().await.expect("Could not deserialize response body to ErrorResponse").error,
        "Missing token".to_string()
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!("{JWT_COOKIE_NAME}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/"),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.json::<ErrorResponse>().await.expect("Could not deserialize response body to ErrorResponse").error,
        "Invalid token".to_string()
    );

    app.clean_up().await;
}
