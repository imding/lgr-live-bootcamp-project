use axum::{http::StatusCode, response::IntoResponse};

pub async fn verify_2fa() -> impl IntoResponse {
    println!("/verify-2fa");
    StatusCode::OK.into_response()
}
