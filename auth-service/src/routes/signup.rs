use axum::{http::StatusCode, response::IntoResponse};

pub async fn signup() -> impl IntoResponse {
    println!("/signup");
    StatusCode::OK.into_response()
}
