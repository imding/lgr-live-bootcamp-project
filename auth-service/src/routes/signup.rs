use {
    crate::{
        app_state::AppState,
        domain::{error::AuthAPIError, user::User},
    },
    axum::{extract::State, http::StatusCode, response::IntoResponse, Json},
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
) -> Result<impl IntoResponse, AuthAPIError> {
    println!("/signup");
    let email = request.email.trim();
    let password = request.password.trim();

    if email.is_empty() || !email.contains("@") || password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    if state.user_store.get_user(email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let user = User::new(email, password, request.requires_2fa);

    if state.user_store.add_user(user).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            message: "User created successfully!".to_string(),
        }),
    ))
}
