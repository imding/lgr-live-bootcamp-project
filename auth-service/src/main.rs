use {
    auth_service::{
        Application,
        app_state::AppState,
        services::{HashmapUserStore, HashsetBannedTokenStore},
        utils::constants::prod,
    },
    std::sync::Arc,
};

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let banned_token_store = HashsetBannedTokenStore::default();
    let app_state = AppState::new(Arc::new(banned_token_store), Arc::new(user_store));
    let app = Application::build(app_state, prod::APP_ADDRESS).await.expect("Failed to build app.");

    app.run().await.expect("Failed to run app.")
}
