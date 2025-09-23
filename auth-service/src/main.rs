use {
    auth_service::{
        Application,
        app_state::AppState,
        get_postgres_pool, get_redis_client,
        services::{MockEmailClient, PostgresUserStore, RedisBannedTokenStore, RedisTwoFactorStore},
        utils::{
            constants::{DATABASE_URL, REDIS_HOST_NAME, prod},
            tracing::init_tracing,
        },
    },
    color_eyre::install,
    redis::Connection as RedisConnection,
    sqlx::{PgPool, migrate},
    std::sync::Arc,
};

#[tokio::main]
async fn main() {
    install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialise tracing");

    let pool = configure_postgresql().await;
    let user_store = PostgresUserStore::new(pool);
    let banned_token_store = RedisBannedTokenStore::new(configure_redis());
    let two_factor_store = RedisTwoFactorStore::new(configure_redis());
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

fn configure_redis() -> RedisConnection {
    get_redis_client(REDIS_HOST_NAME.as_str())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
