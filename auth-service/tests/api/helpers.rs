use {
    auth_service::Application,
    reqwest::{Client, ClientBuilder, Response, header::COOKIE},
    serde::Serialize,
    serde_json::json,
    uuid::Uuid,
};

pub struct TestApp {
    pub address: String,
    pub http_client: Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let Ok(http_client) = ClientBuilder::new().build()
        else {
            panic!("Failed to build reqwest client.")
        };

        Self {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(&json!(body))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login(&self, email: &str, password: &str) -> Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(&json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self, jwt: &str) -> Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .header(COOKIE, json!({ "jwt": jwt }).to_string())
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa(&self, email: &str, attempt_id: &str, code: &str) -> Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(&json!({
                "email": email,
                "loginAttemptId": attempt_id,
                "2FACode": code
            }))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token(&self, token: &str) -> Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(&json!({ "token": token }))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
