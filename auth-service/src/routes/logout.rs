use {
    crate::{
        app_state::AppState,
        domain::error::AuthAPIError,
        utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    },
    axum::{extract::State, http::StatusCode, response::IntoResponse},
    axum_extra::extract::{CookieJar, cookie::Cookie},
    secrecy::SecretBox,
    tracing::instrument,
};

#[instrument(name = "Logout", skip_all)]
pub async fn logout(state: State<AppState>, jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let Some(cookie) = jar.get(JWT_COOKIE_NAME)
    else {
        return Err(AuthAPIError::MissingToken);
    };
    let token = cookie.value().to_owned();

    if token.is_empty() {
        return Err(AuthAPIError::MissingToken);
    }

    if validate_token(Some(state.banned_token_store.clone()), &SecretBox::new(Box::new(token.to_owned())))
        .await
        .is_err()
    {
        return Err(AuthAPIError::InvalidToken);
    }

    if let Err(e) = state.banned_token_store.register(vec![&SecretBox::new(Box::new(token.clone()))]).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    Ok((jar.remove(Cookie::from((JWT_COOKIE_NAME, token))), StatusCode::OK))
}
