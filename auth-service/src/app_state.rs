use {
    crate::domain::data_stores::{BannedTokenStore, TwoFactorStore, UserStore},
    std::sync::Arc,
};

pub type UserStoreType = Arc<dyn UserStore>;
pub type BannedTokenStoreType = Arc<dyn BannedTokenStore>;
pub type TwoFactorStoreType = Arc<dyn TwoFactorStore>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_store: TwoFactorStoreType,
}

impl AppState {
    pub fn new(
        banned_token_store: BannedTokenStoreType,
        user_store: UserStoreType,
        two_factor_store: TwoFactorStoreType,
    ) -> Self {
        Self { banned_token_store, user_store, two_factor_store }
    }
}
