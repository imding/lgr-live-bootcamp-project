use {
    crate::domain::{
        data_stores::{LoginAttemptId, TwoFactorCode, TwoFactorStore, TwoFactorStoreError},
        email::Email,
    },
    std::collections::HashMap,
    tokio::sync::RwLock,
};

#[derive(Default)]
pub struct HashmapTwoFactorStore {
    codes: RwLock<HashMap<Email, (LoginAttemptId, TwoFactorCode)>>,
}

#[async_trait::async_trait]
impl TwoFactorStore for HashmapTwoFactorStore {
    async fn add_code(
        &self,
        email: Email,
        attempt_id: LoginAttemptId,
        code: TwoFactorCode,
    ) -> Result<(), TwoFactorStoreError> {
        self.codes.write().await.insert(email, (attempt_id, code));

        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorStoreError> {
        let mut codes = self.codes.write().await;

        match codes.remove(email) {
            Some(_) => Ok(()),
            None => Err(TwoFactorStoreError::LoginAttemptIdNotFound),
        }
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFactorCode), TwoFactorStoreError> {
        let codes = self.codes.read().await;
        match codes.get(email) {
            Some(value) => Ok((value.0.clone(), value.1.clone())),
            None => Err(TwoFactorStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_code() {
        let store = HashmapTwoFactorStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let code = TwoFactorCode::default();
        let attempt_id = LoginAttemptId::default();

        assert_eq!(store.add_code(email, attempt_id, code).await, Ok(()));
    }

    #[tokio::test]
    async fn test_remove_code() {
        let store = HashmapTwoFactorStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let code = TwoFactorCode::default();
        let attempt_id = LoginAttemptId::default();

        assert_eq!(store.remove_code(&email).await, Err(TwoFactorStoreError::LoginAttemptIdNotFound));
        assert_eq!(store.add_code(email.clone(), attempt_id, code).await, Ok(()));
        assert_eq!(store.remove_code(&email).await, Ok(()));
    }

    #[tokio::test]
    async fn test_get_code() {
        let store = HashmapTwoFactorStore::default();
        let email = Email::parse("me@null.computer").unwrap();
        let code = TwoFactorCode::default();
        let attempt_id = LoginAttemptId::default();

        assert_eq!(store.get_code(&email).await, Err(TwoFactorStoreError::LoginAttemptIdNotFound));
        assert_eq!(store.add_code(email.clone(), attempt_id.clone(), code.clone()).await, Ok(()));

        let value = store.get_code(&email).await;

        assert!(value.is_ok());

        let value = value.unwrap();

        assert_eq!(value.0, attempt_id);
        assert_eq!(value.1, code);
    }
}
