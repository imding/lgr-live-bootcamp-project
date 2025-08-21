use axum::{http::StatusCode, response::IntoResponse};

pub async fn login() -> impl IntoResponse {
    println!("/login");
    StatusCode::OK.into_response()
}
