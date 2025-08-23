use {crate::services::HashmapUserStore, std::sync::Arc, tokio::sync::RwLock};

pub type UserStoreType = Arc<RwLock<HashmapUserStore>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
