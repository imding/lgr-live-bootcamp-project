use {
    crate::{
        app_state::AppState,
        domain::error::AuthAPIError,
        utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    },
    axum::{extract::State, http::StatusCode, response::IntoResponse},
    axum_extra::extract::{CookieJar, cookie::Cookie},
};

pub async fn logout(state: State<AppState>, jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    println!("/logout");
    let Some(cookie) = jar.get(JWT_COOKIE_NAME)
    else {
        return Err(AuthAPIError::MissingToken);
    };
    let token = cookie.value().to_owned();

    if token.is_empty() {
        return Err(AuthAPIError::MissingToken);
    }

    if validate_token(Some(state.banned_token_store.clone()), &token).await.is_err() {
        return Err(AuthAPIError::InvalidToken);
    }

    if state.banned_token_store.register(vec![&token]).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    Ok((jar.remove(Cookie::from((JWT_COOKIE_NAME, token))), StatusCode::OK))
}
