use {
    crate::domain::data_stores::{BannedTokenStore, UserStore},
    std::sync::Arc,
};

pub type UserStoreType = Arc<dyn UserStore>;
pub type BannedTokenStoreType = Arc<dyn BannedTokenStore>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
}

impl AppState {
    pub fn new(banned_token_store: BannedTokenStoreType, user_store: UserStoreType) -> Self {
        Self { banned_token_store, user_store }
    }
}
