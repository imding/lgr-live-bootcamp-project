use {
    crate::{
        app_state::AppState,
        domain::{
            data_stores::{LoginAttemptId, TwoFactorCode},
            email::Email,
            error::AuthAPIError,
            password::Password,
        },
        utils::auth::generate_auth_cookie,
    },
    axum::{
        Json,
        extract::{State, rejection::JsonRejection},
        http::StatusCode,
        response::IntoResponse,
    },
    axum_extra::extract::CookieJar,
    secrecy::{ExposeSecret, SecretBox},
    serde::{Deserialize, Serialize},
    tracing::instrument,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: SecretBox<String>,
    pub password: SecretBox<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth(RegularAuthResponse),
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegularAuthResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: LoginAttemptId,
}

#[instrument(name = "Signup", skip_all)]
pub async fn login(
    state: State<AppState>,
    jar: CookieJar,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let Json(request) = match payload {
        Ok(request_json) => request_json,
        Err(rejection) => {
            let message = rejection.body_text();

            return Ok((jar, (rejection.status(), Json(LoginResponse::RegularAuth(RegularAuthResponse { message })))));
        }
    };
    let Ok(email) = Email::parse(&request.email)
    else {
        return Err(AuthAPIError::InvalidCredentials);
    };
    let (user, password) = match (state.user_store.get_user(&email).await, Password::parse(&request.password)) {
        (Ok(user), Ok(password)) => (user, password),
        _ => return Err(AuthAPIError::InvalidCredentials),
    };

    if user.verify_password_hash(password.as_ref()).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let (status, response) = (match user.requires_2fa {
        true => handle_2fa(&email, &state).await,
        false => handle_no_2fa().await,
    })
    .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    if status != StatusCode::OK {
        return Ok((jar, (status, Json(response))));
    }

    let auth_cookie = generate_auth_cookie(&email).map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    Ok((jar.add(auth_cookie), (status, Json(response))))
}

#[instrument(name = "Handle 2FA", skip_all)]
async fn handle_2fa(email: &Email, state: &AppState) -> Result<(StatusCode, LoginResponse), AuthAPIError> {
    let attempt_id = LoginAttemptId::default();
    let code = TwoFactorCode::default();

    state
        .two_factor_store
        .add_code(email.clone(), attempt_id.clone(), code.clone())
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    state
        .email_client
        .send_email(email, "Your 2FA", code.as_ref().expose_secret())
        .await
        .map_err(AuthAPIError::UnexpectedError)?;

    Ok((
        StatusCode::PARTIAL_CONTENT,
        LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: attempt_id,
        }),
    ))
}

#[instrument(name = "Handle no 2FA", skip_all)]
async fn handle_no_2fa() -> Result<(StatusCode, LoginResponse), AuthAPIError> {
    return Ok((
        StatusCode::OK,
        LoginResponse::RegularAuth(RegularAuthResponse { message: "User logged in successfully!".to_string() }),
    ));
}
