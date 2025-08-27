pub mod app_state;
mod domain;
pub mod routes;
pub mod services;

use {
    crate::{
        domain::error::AuthAPIError,
        routes::{login, logout, signup, verify_2fa, verify_token}
    },
    app_state::AppState,
    axum::{
        http::StatusCode,
        Json,
        Router,
        response::{IntoResponse, Response},
        routing::post,
        serve::Serve,
    },
    serde::{Deserialize, Serialize},
    std::error::Error,
    tower_http::services::ServeDir,
};

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string()
        });

        (status, body).into_response()
    }
}
