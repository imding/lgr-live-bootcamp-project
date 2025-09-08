use {
    auth_service::{
        Application,
        app_state::{AppState, BannedTokenStoreType, TwoFactorStoreType},
        services::{HashmapTwoFactorStore, HashmapUserStore, HashsetBannedTokenStore},
        utils::constants::test,
    },
    reqwest::{
        Client, ClientBuilder, Response, Url,
        cookie::{CookieStore, Jar},
        header::COOKIE,
    },
    serde::Serialize,
    serde_json::json,
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
        let user_store = Arc::new(HashmapUserStore::default());
        let banned_token_store = Arc::new(HashsetBannedTokenStore::default());
        let two_factor_store = Arc::new(HashmapTwoFactorStore::default());
        let app_state = AppState::new(banned_token_store.clone(), user_store, two_factor_store.clone());
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

    pub async fn post_verify_2fa(&self, email: &str, attempt_id: &str, code: &str) -> Response {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(&json!({
                "email": email,
                "loginAttemptId": attempt_id,
                "2FACode": code
            }))
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
