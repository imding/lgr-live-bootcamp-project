use {crate::domain::data_stores::UserStore, std::sync::Arc};

pub type UserStoreType = Arc<dyn UserStore>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
