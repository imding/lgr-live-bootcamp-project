use {
    axum::{Json, http::StatusCode, response::IntoResponse},
    serde::Deserialize,
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

pub async fn signup(Json(_request): Json<SignupRequest>) -> impl IntoResponse {
    println!("/signup");
    StatusCode::OK.into_response()
}
