use {
    auth_service::{Application, app_state::AppState, services::HashmapUserStore, utils::constants::prod},
    std::sync::Arc,
};

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let app_state = AppState::new(Arc::new(user_store));
    let app = Application::build(app_state, prod::APP_ADDRESS).await.expect("Failed to build app.");

    app.run().await.expect("Failed to run app.")
}
