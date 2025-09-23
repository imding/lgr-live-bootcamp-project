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
    tracing::instrument,
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

#[instrument(name = "Verify two factor", skip_all)]
pub async fn verify_2fa(
    state: State<AppState>,
    jar: CookieJar,
    ValidatedJson(request): ValidatedJson<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let Ok((attempt_id, code)) = state.two_factor_store.get_code(&request.email).await
    else {
        return Err(AuthAPIError::IncorrectCredentials);
    };

    if request.login_attempt_id != attempt_id || request.two_factor_code != code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let auth_cookie = match generate_auth_cookie(&request.email) {
        Ok(auth_cookie) => auth_cookie,
        Err(e) => return Err(AuthAPIError::UnexpectedError(e)),
    };

    if let Err(e) = state.two_factor_store.remove_code(&request.email).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    };

    Ok((jar.add(auth_cookie), StatusCode::OK))
}
