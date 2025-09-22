use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{domain::email::Email, utils::auth::generate_auth_token},
    serde_json::json,
};

#[tokio::test]
async fn should_return_200_if_malformed_input() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(&email).unwrap();
    let token = generate_auth_token(&email).unwrap();
    let response = app.post_verify_token(&json!({ "token": token })).await;

    assert_eq!(response.status().as_u16(), 200);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(&email).unwrap();
    let token = generate_auth_token(&email).unwrap();

    assert!(app.banned_token_store.register(vec![&token]).await.is_ok());

    let response = app.post_verify_token(&json!({ "token": token })).await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_input() {
    let mut app = TestApp::new().await;
    let response = app.post_verify_token(&json!({ "token": "abcd.efgh.ijkl" })).await;

    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let response = app.post_verify_token(&json!({ "token": "abcd.efgh" })).await;

    assert_eq!(response.status().as_u16(), 422);

    app.clean_up().await;
}
