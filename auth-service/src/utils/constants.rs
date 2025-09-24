pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const RESEND_SENDER_API_KEY_ENV_VAR: &str = "RESEND_SENDER_API_KEY";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";

    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "sd@vitalstudio.net";
        pub const TIMEOUT: Duration = Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";

    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@null.computer";
        pub const TIMEOUT: Duration = Duration::from_secs(200);
    }
}

use {dotenvy::dotenv, lazy_static::lazy_static, secrecy::SecretBox, std::env::var};

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

lazy_static! {
    pub static ref JWT_SECRET: SecretBox<String> = set_token();
    pub static ref DATABASE_URL: SecretBox<String> = set_database_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
    pub static ref RESEND_SENDER_API_KEY: SecretBox<String> = set_resend_token();
}

fn set_token() -> SecretBox<String> {
    dotenv().ok();

    let secret = var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");

    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }

    SecretBox::new(Box::new(secret))
}

fn set_database_url() -> SecretBox<String> {
    dotenv().ok();

    let secret = var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set.");

    if secret.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }

    SecretBox::new(Box::new(secret))
}

fn set_redis_host() -> String {
    dotenv().ok();

    var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

fn set_resend_token() -> SecretBox<String> {
    dotenv().ok();

    let secret = var(env::RESEND_SENDER_API_KEY_ENV_VAR).expect("RESEND_SENDER_API_KEY must be set.");

    if secret.is_empty() {
        panic!("RESEND_SENDER_API_KEY must not be empty.");
    }

    SecretBox::new(Box::new(secret))
}
