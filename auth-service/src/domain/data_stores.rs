use {
    crate::domain::{
        email::Email,
        password::Password,
        user::{User, UserRow},
    },
    color_eyre::eyre::Report,
    rand::{Rng, rng},
    secrecy::{ExposeSecret, SecretBox},
    serde::{Deserialize, Deserializer, Serialize},
    thiserror::Error,
    uuid::Uuid,
};

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug, Error)]
pub enum TwoFactorStoreError {
    #[error("Login attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug)]
pub struct LoginAttemptId(SecretBox<String>);

#[derive(Debug)]
pub struct TwoFactorCode(SecretBox<String>);

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<UserRow, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn register(&self, tokens: Vec<&SecretBox<String>>) -> Result<(), BannedTokenStoreError>;
    async fn check(&self, token: &SecretBox<String>) -> Result<bool, BannedTokenStoreError>;
}

#[async_trait::async_trait]
pub trait TwoFactorStore: Send + Sync {
    async fn add_code(
        &self,
        email: Email,
        attempt_id: LoginAttemptId,
        code: TwoFactorCode,
    ) -> Result<(), TwoFactorStoreError>;

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorStoreError>;

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFactorCode), TwoFactorStoreError>;
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::InvalidCredentials, Self::InvalidCredentials) |
                (Self::UserAlreadyExists, Self::UserAlreadyExists) |
                (Self::UserNotFound, Self::UserNotFound) |
                (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (Self::UnexpectedError(_), Self::UnexpectedError(_)))
    }
}

impl PartialEq for TwoFactorStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound) |
                (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

impl LoginAttemptId {
    pub fn parse(maybe_uuid: &str) -> Result<Self, String> {
        match Uuid::parse_str(maybe_uuid) {
            Ok(uuid) => Ok(Self(SecretBox::new(Box::new(uuid.to_string())))),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl AsRef<SecretBox<String>> for LoginAttemptId {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(SecretBox::new(Box::new(Uuid::new_v4().to_string())))
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Clone for LoginAttemptId {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl Serialize for LoginAttemptId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("LoginAttemptId", self.0.expose_secret())
    }
}

impl<'a> Deserialize<'a> for LoginAttemptId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let maybe_id = String::deserialize(deserializer)?;
        LoginAttemptId::parse(&maybe_id).map_err(serde::de::Error::custom)
    }
}

impl TwoFactorCode {
    pub fn parse(code: &str) -> Result<Self, String> {
        if code.len() != 6 {
            return Err(String::from("Invalid login attempt ID"));
        }

        Ok(TwoFactorCode(SecretBox::new(Box::new(code.to_string()))))
    }
}

impl AsRef<SecretBox<String>> for TwoFactorCode {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

impl Default for TwoFactorCode {
    fn default() -> Self {
        Self(SecretBox::new(Box::new(rng().random_range(100000..999999).to_string())))
    }
}

impl PartialEq for TwoFactorCode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Clone for TwoFactorCode {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl<'a> Deserialize<'a> for TwoFactorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let maybe_code = String::deserialize(deserializer)?;
        TwoFactorCode::parse(&maybe_code).map_err(serde::de::Error::custom)
    }
}
