use crate::helpers::TestApp;

#[tokio::test]
async fn logout() {
    let app = TestApp::new().await;
    let response = app.post_logout("jwt_token").await;

    assert_eq!(response.status().as_u16(), 200);
}
