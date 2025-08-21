use crate::helpers::TestApp;

#[tokio::test]
async fn verify_2fa() {
    let app = TestApp::new().await;
    let response = app.post_verify_2fa(
        "me@null.computer",
        "attemp_0",
        "123456"
    ).await;

    assert_eq!(response.status().as_u16(), 200);
}
