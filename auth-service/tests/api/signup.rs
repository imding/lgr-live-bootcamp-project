use crate::helpers::TestApp;

#[tokio::test]
async fn signup() {
    let app = TestApp::new().await;
    let response = app.post_signup(
        "me@null.computer",
        "abc123",
        true
    ).await;

    assert_eq!(response.status().as_u16(), 200);
}
