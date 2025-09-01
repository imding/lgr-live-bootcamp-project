use {
    crate::{
        domain::error::AuthAPIError,
        utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    },
    axum::{http::StatusCode, response::IntoResponse},
    axum_extra::extract::{CookieJar, cookie::Cookie},
};

pub async fn logout(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    println!("/logout");
    let Some(cookie) = jar.get(JWT_COOKIE_NAME)
    else {
        return Err(AuthAPIError::MissingToken);
    };
    let token = cookie.value().to_owned();

    if token.is_empty() {
        return Err(AuthAPIError::MissingToken);
    }

    if validate_token(&token).await.is_err() {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok((jar.remove(Cookie::from((JWT_COOKIE_NAME, token))), StatusCode::OK))
}
