use {
    auth_service::{Application, app_state::AppState, services::HashmapUserStore},
    std::sync::Arc,
    tokio::sync::RwLock,
};

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let app_state = AppState::new(Arc::new(RwLock::new(user_store)));
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app.");

    app.run().await.expect("Failed to run app.")
}
