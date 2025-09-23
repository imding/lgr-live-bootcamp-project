use {
    crate::domain::{
        data_stores::{LoginAttemptId, TwoFactorCode, TwoFactorStore, TwoFactorStoreError},
        email::Email,
    },
    redis::{Connection, TypedCommands},
    serde::{Deserialize, Serialize},
    serde_json::{from_str, to_string},
    tokio::sync::RwLock,
};

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FACTOR_PREFIX: &str = "two_factor:";

#[derive(Serialize, Deserialize)]
struct TwoFactorTuple(pub String, pub String);

pub struct RedisTwoFactorStore {
    connection: RwLock<Connection>,
}

impl RedisTwoFactorStore {
    pub fn new(connection: Connection) -> Self {
        Self { connection: RwLock::new(connection) }
    }
}

#[async_trait::async_trait]
impl TwoFactorStore for RedisTwoFactorStore {
    async fn add_code(
        &self,
        email: Email,
        attempt_id: LoginAttemptId,
        code: TwoFactorCode,
    ) -> Result<(), TwoFactorStoreError> {
        let Ok(tuple_string) = to_string(&TwoFactorTuple(attempt_id.as_ref().to_string(), code.as_ref().to_string()))
        else {
            return Err(TwoFactorStoreError::UnexpectedError);
        };
        let mut connection = self.connection.write().await;

        if connection.set_ex(get_key(&email), tuple_string, TEN_MINUTES_IN_SECONDS).is_err() {
            return Err(TwoFactorStoreError::UnexpectedError);
        }

        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorStoreError> {
        let mut connection = self.connection.write().await;

        if connection.del(get_key(email)).is_err() {
            return Err(TwoFactorStoreError::UnexpectedError);
        }

        Ok(())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFactorCode), TwoFactorStoreError> {
        let mut connection = self.connection.write().await;
        let Ok(maybe_tuple_string) = connection.get(get_key(email))
        else {
            return Err(TwoFactorStoreError::UnexpectedError);
        };
        let Some(tuple_string) = maybe_tuple_string
        else {
            return Err(TwoFactorStoreError::LoginAttemptIdNotFound);
        };
        let Ok(tuple) = from_str::<TwoFactorTuple>(&tuple_string)
        else {
            return Err(TwoFactorStoreError::UnexpectedError);
        };
        let Ok(attempt_id) = LoginAttemptId::parse(&tuple.0)
        else {
            return Err(TwoFactorStoreError::UnexpectedError);
        };
        let Ok(code) = TwoFactorCode::parse(&tuple.1)
        else {
            return Err(TwoFactorStoreError::UnexpectedError);
        };

        Ok((attempt_id, code))
    }
}

fn get_key(email: &Email) -> String {
    format!("{TWO_FACTOR_PREFIX} {}", email.as_ref())
}
