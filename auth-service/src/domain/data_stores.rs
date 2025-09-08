use {
    crate::domain::{email::Email, password::Password, user::User},
    rand::{Rng, rng},
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn register(&self, tokens: Vec<&str>);
    async fn check(&self, token: &str) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    InvalidCredentials,
    UserAlreadyExists,
    UserNotFound,
    UnexpectedError,
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

#[derive(Debug, PartialEq)]
pub enum TwoFactorStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LoginAttemptId(String);

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFactorCode(String);

impl LoginAttemptId {
    pub fn parse(code: &str) -> Result<Self, String> {
        match Uuid::parse_str(code) {
            Ok(uuid) => Ok(Self(uuid.to_string())),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl TwoFactorCode {
    pub fn parse(code: &str) -> Result<Self, String> {
        if code.len() != 6 {
            return Err(String::from("Invalid login attempt ID"));
        }

        Ok(TwoFactorCode(code.to_string()))
    }
}

impl Default for TwoFactorCode {
    fn default() -> Self {
        Self(rng().random_range(100000..999999).to_string())
    }
}
