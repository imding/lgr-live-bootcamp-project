use {
    crate::helpers::{TestApp, get_random_email},
    serde_json::json,
};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        json!({
            "password": "abc123",
            "require2FA": true
        }),
        json!({
            "email": random_email,
            "required2FA": true
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
