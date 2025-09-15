use {
    crate::domain::{
        data_stores::{LoginAttemptId, TwoFactorCode},
        email::Email,
        error::AuthAPIError,
    },
    axum::{
        Json,
        extract::{FromRequest, Request, rejection::JsonRejection::JsonDataError},
        http::StatusCode,
        response::IntoResponse,
    },
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
    ValidatedJson(request): ValidatedJson<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    println!("/verify-2fa");

    Ok(StatusCode::OK)
}
