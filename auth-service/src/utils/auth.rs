use {
    crate::{
        app_state::BannedTokenStoreType,
        domain::email::Email,
        utils::constants::{JWT_COOKIE_NAME, JWT_SECRET},
    },
    axum_extra::extract::cookie::{Cookie, SameSite},
    chrono::{Duration, Utc},
    color_eyre::{
        Report,
        eyre::{Context, ContextCompat as _, Result, eyre},
    },
    jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError},
    secrecy::{ExposeSecret, SecretBox},
    serde::{Deserialize, Serialize},
    thiserror::Error,
    tracing::instrument,
};

pub const TOKEN_TTL_SECONDS: i64 = 600;

#[derive(Debug, Error)]
pub enum GenerateTokenError {
    #[error("Token error")]
    TokenError(JwtError),
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug)]
pub enum ValidateTokenError {
    TokenError(JwtError),
    BannedToken,
    UnexpectedError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub exp: usize,
    pub sub: String,
}

#[instrument(name = "Generate auth cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;

    Ok(create_auth_cookie(token))
}

#[instrument(name = "Validate token", skip_all)]
pub async fn validate_token(
    store: Option<BannedTokenStoreType>,
    token: &SecretBox<String>,
) -> Result<Claims, ValidateTokenError> {
    if let Some(store) = store {
        let Ok(exists) = store.check(token).await
        else {
            return Err(ValidateTokenError::UnexpectedError);
        };

        if exists {
            return Err(ValidateTokenError::BannedToken);
        }
    }

    match decode::<Claims>(
        token.expose_secret(),
        &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    {
        Ok(claims) => Ok(claims),
        Err(error) => Err(ValidateTokenError::TokenError(error)),
    }
}

#[instrument(name = "Create auth cookie", skip_all)]
fn create_auth_cookie(token: SecretBox<String>) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token.expose_secret().to_owned()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    cookie
}

#[instrument(name = "Generate auth token", skip_all)]
pub fn generate_auth_token(email: &Email) -> Result<SecretBox<String>> {
    let delta = Duration::try_seconds(TOKEN_TTL_SECONDS).wrap_err("Failed to create 10 minutes time delta")?;
    let exp =
        Utc::now().checked_add_signed(delta).ok_or(eyre!("Failed to add 10 minutes to current time"))?.timestamp();
    let exp: usize = exp.try_into().wrap_err(format!("Failed to cast exp time to usize. exp time: {exp}"))?;
    let sub = email.as_ref().expose_secret().to_owned();
    let claims = Claims { sub, exp };

    create_token(&claims)
}

#[instrument(name = "Create token", skip_all)]
fn create_token(claims: &Claims) -> Result<SecretBox<String>> {
    match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()))
        .wrap_err("Failed to create token")
    {
        Ok(token) => Ok(SecretBox::new(Box::new(token))),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use {super::*, secrecy::SecretBox};

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse(&SecretBox::new(Box::new("test@example.com".to_string()))).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();

        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(SecretBox::new(Box::new(token.clone())));

        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse(&SecretBox::new(Box::new("test@example.com".to_string()))).unwrap();
        let result = generate_auth_token(&email).unwrap();

        assert_eq!(result.expose_secret().split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse(&SecretBox::new(Box::new("test@example.com".to_string()))).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token(None, &token).await.unwrap();

        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token(None, &SecretBox::new(Box::new(token))).await;

        assert!(result.is_err());
    }
}
