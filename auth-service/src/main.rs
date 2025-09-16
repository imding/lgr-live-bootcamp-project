use {
    auth_service::{
        Application,
        app_state::AppState,
        get_postgres_pool,
        services::{HashmapTwoFactorStore, HashmapUserStore, HashsetBannedTokenStore, MockEmailClient},
        utils::constants::{DATABASE_URL, prod},
    },
    sqlx::{PgPool, migrate},
    std::sync::Arc,
};

#[tokio::main]
async fn main() {
    let _ = configure_postgresql().await;
    let user_store = HashmapUserStore::default();
    let banned_token_store = HashsetBannedTokenStore::default();
    let two_factor_store = HashmapTwoFactorStore::default();
    let app_state = AppState::new(
        Arc::new(banned_token_store),
        Arc::new(user_store),
        Arc::new(two_factor_store),
        Arc::new(MockEmailClient),
    );
    let app = Application::build(app_state, prod::APP_ADDRESS).await.expect("Failed to build app.");

    app.run().await.expect("Failed to run app.")
}

async fn configure_postgresql() -> PgPool {
    println!("Configuring database...");
    let pool = get_postgres_pool(&DATABASE_URL).await.expect("Failed to create Postgres connection pool");

    println!("Migrating database...");
    migrate!().run(&pool).await.expect("Failed to run migrations");

    pool
}
