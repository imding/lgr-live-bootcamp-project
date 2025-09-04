use crate::domain::{email::Email, password::Password, user::User};

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
