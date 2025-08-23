use {crate::domain::user::User, std::collections::HashMap};

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);

        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        let Some(maybe_user) = self.users.get(email)
        else {
            return Err(UserStoreError::UserNotFound);
        };

        return Ok(maybe_user);
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;

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
        let mut store = HashmapUserStore::default();
        let user = User::new("me@null.computer", "abc123", true);

        assert_eq!(store.add_user(user.clone()), Ok(()));
        assert_eq!(store.add_user(user), Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("me@null.computer", "abc123", true);

        assert_eq!(store.add_user(user.clone()), Ok(()));
        assert_eq!(store.get_user("me@null.computer"), Ok(&user));
        assert_eq!(
            store.get_user("you@null.computer"),
            Err(UserStoreError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = "me@null.computer";
        let password = "abc123";
        let user = User::new(email, password, true);

        assert_eq!(
            store.validate_user(email, password),
            Err(UserStoreError::UserNotFound)
        );
        assert_eq!(store.add_user(user.clone()), Ok(()));
        assert_eq!(store.validate_user(email, password), Ok(()));
        assert_eq!(
            store.validate_user(email, "xyz123"),
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
