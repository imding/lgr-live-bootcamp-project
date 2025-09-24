use {
    auth_service::{
        Application,
        app_state::{AppState, BannedTokenStoreType, TwoFactorStoreType},
        get_postgres_pool, get_redis_client,
        services::{MockEmailClient, PostgresUserStore, RedisBannedTokenStore, RedisTwoFactorStore},
        utils::constants::{DATABASE_URL, REDIS_HOST_NAME, test},
    },
    redis::Connection as RedisConnection,
    reqwest::{
        Client, ClientBuilder, Response, Url,
        cookie::{CookieStore, Jar},
        header::COOKIE,
    },
    secrecy::{ExposeSecret, SecretBox},
    serde::Serialize,
    sqlx::{
        Connection as _, Executor, PgConnection, PgPool, migrate,
        postgres::{PgConnectOptions, PgPoolOptions},
    },
    std::{str::FromStr as _, sync::Arc},
    uuid::Uuid,
};

pub struct TestApp {
    pub address: String,
    pub banned_token_store: BannedTokenStoreType,
    cleaned_up: bool,
    pub cookie_jar: Arc<Jar>,
    pub database_name: String,
    pub http_client: Client,
    pub two_factor_store: TwoFactorStoreType,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("TestApp dropped without clean up.")
        }
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let (pool, database_name) = configure_postgresql().await;
        let user_store = Arc::new(PostgresUserStore::new(pool));
        let banned_token_store = Arc::new(RedisBannedTokenStore::new(configure_redis()));
        let two_factor_store = Arc::new(RedisTwoFactorStore::new(configure_redis()));
        let email_client = Arc::new(MockEmailClient);
        let app_state = AppState::new(banned_token_store.clone(), user_store, two_factor_store.clone(), email_client);
        let app = Application::build(app_state, test::APP_ADDRESS).await.expect("Failed to build app");
        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());
        let cookie_jar = Arc::new(Jar::default());
        let Ok(http_client) = ClientBuilder::new().cookie_provider(Arc::clone(&cookie_jar)).build()
        else {
            panic!("Failed to build reqwest client.")
        };

        Self {
            address,
            banned_token_store,
            cleaned_up: false,
            cookie_jar,
            database_name,
            http_client,
            two_factor_store,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client.get(format!("{}/", &self.address)).send().await.expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> Response {
        let cookies = match Url::parse(&self.address) {
            Ok(url) => match self.cookie_jar.cookies(&url) {
                Some(cookies) => match cookies.to_str() {
                    Ok(cookies) => cookies.to_string(),
                    _ => String::new(),
                },
                _ => String::new(),
            },
            _ => String::new(),
        };

        self.http_client
            .post(format!("{}/logout", &self.address))
            .header(COOKIE, cookies)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.database_name).await;
        self.cleaned_up = true;
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn configure_postgresql() -> (PgPool, String) {
    let url = DATABASE_URL.expose_secret().to_owned();
    let suffix = Uuid::new_v4().to_string();
    let (url, name) = configure_database(&url, &suffix).await;

    (
        get_postgres_pool(&SecretBox::new(Box::new(format!("{url}/{name}"))))
            .await
            .expect("Failed to create Postgres connection pool!"),
        name,
    )
}

async fn configure_database(url: &str, suffix: &str) -> (String, String) {
    let (url, name) = url.rsplit_once('/').unwrap();
    let connection = PgPoolOptions::new()
        .connect(&format!("{url}/postgres"))
        .await
        .expect("Failed to create Postgres connection pool.");

    connection
        .execute(format!(r#"create database "{name}-{suffix}";"#).as_str())
        .await
        .expect("Failed to create database.");

    let name = format!("{name}-{suffix}");
    let connection = PgPoolOptions::new()
        .connect(&format!("{url}/{name}"))
        .await
        .expect("Failed to create Postgres connection pool.");

    migrate!().run(&connection).await.expect("Failed to migrate the database");

    (url.to_string(), name)
}

async fn delete_database(name: &str) {
    let (url, _) = DATABASE_URL.expose_secret().rsplit_once('/').unwrap();
    let options =
        PgConnectOptions::from_str(&format!("{url}/postgres")).expect("Failed to parse PostgreSQL connection string");
    let mut connection = PgConnection::connect_with(&options).await.expect("Failed to connect to Postgres");

    connection
        .execute(
            format!(
                r#"select pg_terminate_backend(pg_stat_activity.pid) from pg_stat_activity
                where
                    pg_stat_activity.datname = '{name}' and
                    pid <> pg_backend_pid();"#
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database");

    connection.execute(format!(r#"drop database "{name}";"#).as_str()).await.expect("Failed to drop the database");
}

fn configure_redis() -> RedisConnection {
    get_redis_client(REDIS_HOST_NAME.as_str())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
