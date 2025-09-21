use {
    auth_service::{
        Application,
        app_state::{AppState, BannedTokenStoreType, TwoFactorStoreType},
        get_postgres_pool,
        services::{HashmapTwoFactorStore, HashsetBannedTokenStore, MockEmailClient, PostgresUserStore},
        utils::constants::{DATABASE_URL, test},
    },
    reqwest::{
        Client, ClientBuilder, Response, Url,
        cookie::{CookieStore, Jar},
        header::COOKIE,
    },
    serde::Serialize,
    sqlx::{Executor, PgPool, migrate, postgres::PgPoolOptions},
    std::sync::Arc,
    uuid::Uuid,
};

pub struct TestApp {
    pub address: String,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_store: TwoFactorStoreType,
    pub cookie_jar: Arc<Jar>,
    pub http_client: Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(PostgresUserStore::new(configure_postgresql().await));
        let banned_token_store = Arc::new(HashsetBannedTokenStore::default());
        let two_factor_store = Arc::new(HashmapTwoFactorStore::default());
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

        Self { address, cookie_jar, banned_token_store, two_factor_store, http_client }
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
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn configure_postgresql() -> PgPool {
    let url = DATABASE_URL.to_owned();
    let suffix = Uuid::new_v4().to_string();

    configure_database(&url, &suffix).await;

    let url = format!("{url}-{suffix}");

    get_postgres_pool(&url).await.expect("Failed to create Postgres connection pool!")
}

async fn configure_database(url: &str, suffix: &str) {
    let (url, name) = url.rsplit_once('/').unwrap();
    let connection = PgPoolOptions::new()
        .connect(&format!("{url}/postgres"))
        .await
        .expect("Failed to create Postgres connection pool.");

    connection
        .execute(format!(r#"create database "{name}-{suffix}";"#).as_str())
        .await
        .expect("Failed to create database.");

    let url = format!("{url}/{name}-{suffix}");
    let connection = PgPoolOptions::new().connect(&url).await.expect("Failed to create Postgres connection pool.");

    migrate!().run(&connection).await.expect("Failed to migrate the database");
}
