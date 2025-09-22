use {
    crate::{
        domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
        utils::auth::TOKEN_TTL_SECONDS,
    },
    redis::{Connection, TypedCommands},
    tokio::sync::RwLock,
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
    async fn check(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut connection = self.connection.write().await;
        let Ok(exists) = connection.exists(get_key(token))
        else {
            return Err(BannedTokenStoreError::UnexpectedError);
        };

        return Ok(exists);
    }

    async fn register(&self, tokens: Vec<&str>) -> Result<(), BannedTokenStoreError> {
        let mut connection = self.connection.write().await;

        for token in tokens {
            let Ok(exists) = connection.exists(get_key(token))
            else {
                return Err(BannedTokenStoreError::UnexpectedError);
            };

            if exists {
                continue;
            }

            if connection.set_ex::<String, bool>(get_key(token), true, TOKEN_TTL_SECONDS as u64).is_err() {
                return Err(BannedTokenStoreError::UnexpectedError);
            }
        }

        Ok(())
    }
}

fn get_key(token: &str) -> String {
    format!("{BANNED_TOKEN_KEY_PREFIX} {token}")
}
