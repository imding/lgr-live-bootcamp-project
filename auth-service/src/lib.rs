pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

use {
    crate::{
        domain::error::AuthAPIError,
        routes::{login, logout, signup, verify_2fa, verify_token},
    },
    app_state::AppState,
    axum::{
        Json, Router,
        http::{Method, StatusCode},
        response::{IntoResponse, Response},
        routing::post,
    },
    serde::{Deserialize, Serialize},
    sqlx::{PgPool, postgres::PgPoolOptions},
    std::{error::Error, io::Error as IoError, time::Duration},
    tokio::net::TcpListener,
    tower_http::{cors::CorsLayer, services::ServeDir},
};

pub struct Application {
    pub address: String,
    listener: TcpListener,
    router: Router,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let cors = CorsLayer::new().allow_methods([Method::GET, Method::POST]).allow_credentials(true).allow_origin([
            "http://localhost:8000".parse()?,
            // "http://[]:8000".parse()?
        ]);
        let router = Router::new()
            .fallback_service(ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        Ok(Self { address, listener, router })
    }

    pub async fn run(self) -> Result<(), IoError> {
        println!("listening on {}", &self.address);
        Ok(axum::serve(self.listener, self.router).await?)
    }
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Incorrect credentials"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::MalformedToken => (StatusCode::UNPROCESSABLE_ENTITY, "Malformed token"),
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };
        let body = Json(ErrorResponse { error: error_message.to_string() });

        (status, body).into_response()
    }
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(5).connect(url).await
}
