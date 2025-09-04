use {crate::domain::data_stores::BannedTokenStore, std::collections::HashSet, tokio::sync::RwLock};

pub struct HashsetBannedTokenStore {
    tokens: RwLock<HashSet<String>>,
}

impl Default for HashsetBannedTokenStore {
    fn default() -> Self {
        Self { tokens: RwLock::new(HashSet::new()) }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn register(&self, tokens: Vec<&str>) {
        let mut store_tokens = self.tokens.write().await;

        for token in tokens {
            if store_tokens.contains(token) {
                continue;
            }

            store_tokens.insert(token.to_string());
        }
    }

    async fn check(&self, token: &str) -> bool {
        let store_tokens = self.tokens.read().await;

        return store_tokens.contains(token);
    }
}
