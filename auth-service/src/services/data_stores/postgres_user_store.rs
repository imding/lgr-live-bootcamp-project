use {
    crate::domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        password::Password,
        user::{User, UserRow},
    },
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
        let Ok(user) = user.into_row().await
        else {
            return Err(UserStoreError::UnexpectedError);
        };

        if query_as!(
            UserRow,
            r#"insert into users values ($1, $2, $3) returning *;"#,
            user.email,
            user.password_hash,
            user.requires_2fa,
        )
        .fetch_one(&self.pool)
        .await
        .is_err()
        {
            return Err(UserStoreError::UnexpectedError);
        };

        Ok(())
    }

    #[instrument(name = "Get user from database", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<UserRow, UserStoreError> {
        let Ok(user_row) =
            query_as!(UserRow, r#"select * from users where email = $1;"#, email.as_ref()).fetch_one(&self.pool).await
        else {
            return Err(UserStoreError::UnexpectedError);
        };

        Ok(user_row)
    }

    #[instrument(name = "Validate user credentials in database", skip_all)]
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        if user.verify_password_hash(password.as_ref()).await.is_err() {
            return Err(UserStoreError::UnexpectedError);
        }

        Ok(())
    }
}
