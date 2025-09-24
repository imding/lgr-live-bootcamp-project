use {
    crate::{
        app_state::AppState,
        domain::{email::Email, error::AuthAPIError, password::Password, user::User},
    },
    axum::{Json, extract::State, http::StatusCode, response::IntoResponse},
    secrecy::SecretBox,
    serde::{Deserialize, Serialize},
    tracing::instrument,
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: SecretBox<String>,
    pub password: SecretBox<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SignupResponse {
    pub message: String,
}

#[instrument(name = "Signup", skip_all)]
pub async fn signup(
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let Ok(email) = Email::parse(&request.email)
    else {
        return Err(AuthAPIError::InvalidCredentials);
    };
    let Ok(password) = Password::parse(&request.password)
    else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    if state.user_store.get_user(&email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let user = User::new(&email, &password, request.requires_2fa);

    state.user_store.add_user(user).await.map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    Ok((StatusCode::CREATED, Json(SignupResponse { message: "User created successfully!".to_string() })))
}
