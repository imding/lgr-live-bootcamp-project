use {
    crate::domain::{
        data_stores::{UserStore, UserStoreError},
        user::User,
    },
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};

#[derive(Default)]
pub struct HashmapUserStore {
    users: Arc<Mutex<HashMap<String, User>>>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        let Ok(mut users) = self.users.lock() else {
            return Err(UserStoreError::UnexpectedError);
        };

        if users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        users.insert(user.email.clone(), user);

        Ok(())
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        let Ok(users) = self.users.lock() else {
            return Err(UserStoreError::UnexpectedError);
        };

        let Some(maybe_user) = users.get(email) else {
            return Err(UserStoreError::UserNotFound);
        };

        Ok(maybe_user.clone())
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        if user.password != password {
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
        let user = User::new("me@null.computer", "abc123", true);

        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(
            store.add_user(user).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = HashmapUserStore::default();
        let user = User::new("me@null.computer", "abc123", true);

        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(store.get_user("me@null.computer").await, Ok(user));
        assert_eq!(
            store.get_user("you@null.computer").await,
            Err(UserStoreError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let store = HashmapUserStore::default();
        let email = "me@null.computer";
        let password = "abc123";
        let user = User::new(email, password, true);

        assert_eq!(
            store.validate_user(email, password).await,
            Err(UserStoreError::UserNotFound)
        );
        assert_eq!(store.add_user(user.clone()).await, Ok(()));
        assert_eq!(store.validate_user(email, password).await, Ok(()));
        assert_eq!(
            store.validate_user(email, "xyz123").await,
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
