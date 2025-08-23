use {
    crate::{app_state::AppState, domain::user::User},
    axum::{Json, extract::State, http::StatusCode, response::IntoResponse},
    serde::{Deserialize, Serialize},
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SignupResponse {
    pub message: String,
}

pub async fn signup(
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    println!("/signup");
    let mut user_store = state.user_store.write().await;

    user_store
        .add_user(User::new(
            &request.email,
            &request.password,
            request.requires_2fa,
        ))
        .unwrap();

    (
        StatusCode::CREATED,
        Json(SignupResponse {
            message: "User created successfully!".to_string(),
        }),
    )
}
