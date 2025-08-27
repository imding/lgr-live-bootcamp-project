use {
    crate::helpers::{TestApp, get_random_email},
    auth_service::{ErrorResponse, routes::SignupResponse},
    serde_json::json,
};

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let body = &json!({
        "email": random_email,
        "password": "abcd1234",
        "requires2FA": true
    });
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let test_cases = [
        json!({
            "email": "",
            "password": "abcd1234",
            "requires2FA": true
        }),
        json!({
            "email": "me",
            "password": "abcd1234",
            "requires2FA": true
        }),
        json!({
            "email": "me@null.computer",
            "password": "abcd123",
            "requires2FA": true
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;

        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", test_case);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;
    let input = json!({
        "email": get_random_email(),
        "password": "abcd1234",
        "requires2FA": true
    });
    let _ = app.post_signup(&input).await;
    let response = app.post_signup(&input).await;

    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        json!({
            "password": "abcd1234",
            "requires2FA": true
        }),
        json!({
            "email": random_email,
            "requires2FA": true
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {test_case:?}"
        )
    }
}
