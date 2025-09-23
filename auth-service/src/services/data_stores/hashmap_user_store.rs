use {
    crate::domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        password::Password,
        user::{User, UserRow},
    },
    std::collections::HashMap,
    tokio::sync::RwLock,
};

#[derive(Default)]
pub struct HashmapUserStore {
    users: RwLock<HashMap<Email, UserRow>>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        let mut users = self.users.write().await;

        if users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let user_row = match user.into_row().await {
            Ok(v) => v,
            Err(e) => return Err(UserStoreError::UnexpectedError(e)),
        };

        users.insert(user.email.clone(), user_row);

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<UserRow, UserStoreError> {
        let users = self.users.read().await;
        let Some(user) = users.get(email)
        else {
            return Err(UserStoreError::UserNotFound);
        };

        Ok(user.to_owned())
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        if user.verify_password_hash(password.as_ref()).await.is_err() {
            return Err(UserStoreError::InvalidCredentials);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let store = HashmapUserStore::default();
        let user = User::new(&Email::parse("me@null.computer").unwrap(), &Password::parse("abcd1234").unwrap(), true);

        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(store.add_user(user).await, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = HashmapUserStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let password = Password::parse("abcd1234").unwrap();
        let user = User::new(&email, &password, true);

        assert_eq!(store.add_user(user.clone()).await, Ok(()));

        let user_row = store.get_user(&email).await.unwrap();
        assert_eq!(user_row.email, email.as_ref());
        assert!(user_row.verify_password_hash(password.as_ref()).await.is_ok());
        assert!(user_row.requires_2fa);
        assert_eq!(
            store.get_user(&Email::parse("you@null.computer").unwrap()).await,
            Err(UserStoreError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let store = HashmapUserStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let password = Password::parse("abcd1234").unwrap();
        let user = User::new(&email, &password, true);

        assert_eq!(store.validate_user(&email, &password).await, Err(UserStoreError::UserNotFound));
        assert_eq!(store.add_user(user).await, Ok(()));
        assert_eq!(store.validate_user(&email, &password).await, Ok(()));
        assert_eq!(
            store.validate_user(&email, &Password::parse("wxyz7890").unwrap()).await,
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
