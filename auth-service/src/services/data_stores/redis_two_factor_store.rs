use {
    crate::domain::{
        data_stores::{LoginAttemptId, TwoFactorCode, TwoFactorStore, TwoFactorStoreError},
        email::Email,
    },
    redis::{Connection, TypedCommands},
    serde::{Deserialize, Serialize},
    serde_json::{from_str, to_string},
    tokio::sync::RwLock,
    tracing::instrument,
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
    #[instrument(name = "Add code to redis", skip_all)]
    async fn add_code(
        &self,
        email: Email,
        attempt_id: LoginAttemptId,
        code: TwoFactorCode,
    ) -> Result<(), TwoFactorStoreError> {
        let tuple_string = match to_string(&TwoFactorTuple(attempt_id.as_ref().to_string(), code.as_ref().to_string()))
        {
            Ok(string) => string,
            Err(e) => return Err(TwoFactorStoreError::UnexpectedError(e.into())),
        };
        let mut connection = self.connection.write().await;

        if let Err(e) = connection.set_ex(get_key(&email), tuple_string, TEN_MINUTES_IN_SECONDS) {
            return Err(TwoFactorStoreError::UnexpectedError(e.into()));
        }

        Ok(())
    }

    #[instrument(name = "Remove code from redis", skip_all)]
    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorStoreError> {
        let mut connection = self.connection.write().await;

        if let Err(e) = connection.del(get_key(email)) {
            return Err(TwoFactorStoreError::UnexpectedError(e.into()));
        }

        Ok(())
    }

    #[instrument(name = "Get code from redis", skip_all)]
    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFactorCode), TwoFactorStoreError> {
        let mut connection = self.connection.write().await;

        let maybe_tuple_string: Option<String> = match connection.get(get_key(email)) {
            Ok(v) => v,
            Err(e) => return Err(TwoFactorStoreError::UnexpectedError(e.into())),
        };

        let tuple_string = match maybe_tuple_string {
            Some(s) => s,
            None => return Err(TwoFactorStoreError::LoginAttemptIdNotFound),
        };

        let tuple: TwoFactorTuple = match from_str(&tuple_string) {
            Ok(v) => v,
            Err(e) => return Err(TwoFactorStoreError::UnexpectedError(e.into())),
        };

        let attempt_id = match LoginAttemptId::parse(&tuple.0) {
            Ok(v) => v,
            Err(e) => return Err(TwoFactorStoreError::UnexpectedError(color_eyre::eyre::eyre!(e))),
        };

        let code = match TwoFactorCode::parse(&tuple.1) {
            Ok(v) => v,
            Err(e) => return Err(TwoFactorStoreError::UnexpectedError(color_eyre::eyre::eyre!(e))),
        };

        Ok((attempt_id, code))
    }
}

fn get_key(email: &Email) -> String {
    format!("{TWO_FACTOR_PREFIX}{}", email.as_ref())
}
