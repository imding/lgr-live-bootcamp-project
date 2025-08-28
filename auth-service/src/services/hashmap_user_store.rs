use {
    crate::domain::{
        data_stores::{UserStore, UserStoreError}, email::Email, password::Password, user::User
    },
    std::collections::HashMap,
    tokio::sync::RwLock,
};

#[derive(Default)]
pub struct HashmapUserStore {
    users: RwLock<HashMap<Email, User>>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        let mut users = self.users.write().await;

        if users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        users.insert(user.email.clone(), user);

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let users = self.users.read().await;
        let Some(maybe_user) = users.get(email) else {
            return Err(UserStoreError::UserNotFound);
        };

        Ok(maybe_user.clone())
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        if user.password != *password {
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
        let user = User::new(
            &Email::parse("me@null.computer").unwrap(),
            &Password::parse("abcd1234").unwrap(),
            true
        );

        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(
            store.add_user(user).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = HashmapUserStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let user = User::new(
            &email,
            &Password::parse("abcd1234").unwrap(),
            true
        );

        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(store.get_user(&email).await, Ok(user));
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

        assert_eq!(
            store.validate_user(&email, &password).await,
            Err(UserStoreError::UserNotFound)
        );
        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(store.validate_user(&email, &password).await, Ok(()));
        assert_eq!(
            store.validate_user(&email, &Password::parse("wxyz7890").unwrap()).await,
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
