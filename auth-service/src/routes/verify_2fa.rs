use {
    crate::{
        app_state::AppState,
        domain::{
            data_stores::{LoginAttemptId, TwoFactorCode},
            email::Email,
            error::AuthAPIError,
        },
        utils::auth::generate_auth_cookie,
    },
    axum::{
        Json,
        extract::{FromRequest, Request, State, rejection::JsonRejection::JsonDataError},
        http::StatusCode,
        response::IntoResponse,
    },
    axum_extra::extract::CookieJar,
    serde::Deserialize,
};

#[derive(Deserialize)]
pub struct Verify2FARequest {
    email: Email,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: LoginAttemptId,
    #[serde(rename = "2FACode")]
    two_factor_code: TwoFactorCode,
}

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = AuthAPIError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ValidatedJson(value)),
            Err(err) => match err {
                JsonDataError(err) => match err.body_text().contains("missing field") {
                    true => Err(AuthAPIError::MalformedToken),
                    _ => Err(AuthAPIError::InvalidCredentials),
                },
                _ => Err(AuthAPIError::MalformedToken),
            },
        }
    }
}

pub async fn verify_2fa(
    state: State<AppState>,
    jar: CookieJar,
    ValidatedJson(request): ValidatedJson<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    println!("/verify-2fa");

    let Ok((attempt_id, code)) = state.two_factor_store.get_code(&request.email).await
    else {
        return Err(AuthAPIError::IncorrectCredentials);
    };

    if request.login_attempt_id != attempt_id || request.two_factor_code != code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let Ok(auth_cookie) = generate_auth_cookie(&request.email)
    else {
        return Err(AuthAPIError::UnexpectedError);
    };

    if state.two_factor_store.remove_code(&request.email).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    };

    Ok((jar.add(auth_cookie), StatusCode::OK))
}
