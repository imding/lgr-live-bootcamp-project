use {
    crate::{
        domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
        utils::auth::TOKEN_TTL_SECONDS,
    },
    redis::{Connection, TypedCommands},
    tokio::sync::RwLock,
    tracing::instrument,
};

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

pub struct RedisBannedTokenStore {
    connection: RwLock<Connection>,
}

impl RedisBannedTokenStore {
    pub fn new(connection: Connection) -> Self {
        Self { connection: RwLock::new(connection) }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[instrument(name = "Check token in redis", skip_all)]
    async fn check(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut connection = self.connection.write().await;

        match connection.exists(get_key(token)) {
            Ok(exists) => Ok(exists),
            Err(e) => Err(BannedTokenStoreError::UnexpectedError(e.into())),
        }
    }

    #[instrument(name = "Add token to redis", skip_all)]
    async fn register(&self, tokens: Vec<&str>) -> Result<(), BannedTokenStoreError> {
        let mut connection = self.connection.write().await;

        for token in tokens {
            let exists = match connection.exists(get_key(token)) {
                Ok(v) => v,
                Err(e) => return Err(BannedTokenStoreError::UnexpectedError(e.into())),
            };

            if exists {
                continue;
            }

            if let Err(e) = connection.set_ex(get_key(token), true, TOKEN_TTL_SECONDS as u64) {
                return Err(BannedTokenStoreError::UnexpectedError(e.into()));
            }
        }

        Ok(())
    }
}

fn get_key(token: &str) -> String {
    format!("{BANNED_TOKEN_KEY_PREFIX}{token}")
}
