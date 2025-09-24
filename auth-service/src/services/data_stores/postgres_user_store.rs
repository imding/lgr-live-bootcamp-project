use {
    crate::domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        password::Password,
        user::{User, UserRow},
    },
    secrecy::ExposeSecret,
    sqlx::{PgPool, query_as},
    tracing::instrument,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[instrument(name = "Add user to database", skip_all)]
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        let user = user.into_row().await.map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        query_as!(
            UserRow,
            r#"insert into users values ($1, $2, $3) returning *;"#,
            user.email,
            user.password_hash,
            user.requires_2fa,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "Get user from database", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<UserRow, UserStoreError> {
        let user_row = query_as!(UserRow, r#"select * from users where email = $1;"#, email.as_ref().expose_secret())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(user_row)
    }

    #[instrument(name = "Validate user credentials in database", skip_all)]
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        user.verify_password_hash(password.as_ref()).await.map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }
}
