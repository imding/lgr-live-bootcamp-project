use axum::{http::StatusCode, response::IntoResponse};

pub async fn logout() -> impl IntoResponse {
    println!("/logout");
    StatusCode::OK.into_response()
}
