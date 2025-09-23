use {
    crate::domain::{email::Email, password::Password},
    argon2::{Argon2, PasswordHash, PasswordVerifier},
    std::error::Error,
    tokio::task::spawn_blocking,
    tracing::{Span, instrument},
};

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserRow {
    pub email: String,
    pub password_hash: String,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: &Email, password: &Password, requires_2fa: bool) -> Self {
        Self { email: email.clone(), password: password.clone(), requires_2fa }
    }

    #[instrument(name = "Convert user to row", skip_all)]
    pub async fn into_row(&self) -> Result<UserRow, Box<dyn Error + Send + Sync>> {
        let current_span = Span::current();
        let password = self.password.to_owned();
        let password_hash = spawn_blocking(move || current_span.in_scope(|| password.hash())).await??;

        Ok(UserRow { email: self.email.as_ref().to_string(), password_hash, requires_2fa: self.requires_2fa })
    }
}

impl UserRow {
    #[instrument(name = "Verify password hash", skip_all)]
    pub async fn verify_password_hash(&self, target: &str) -> Result<(), Box<dyn Error>> {
        let current_span = Span::current();
        let hash = self.password_hash.to_owned();
        let target = target.to_owned();

        Ok(spawn_blocking(move || {
            current_span.in_scope(|| {
                let hash = PasswordHash::new(&hash)?;

                Argon2::default().verify_password(target.as_bytes(), &hash)
            })
        })
        .await??)
    }
}
