use {
    argon2::{
        Algorithm, Argon2, Params, PasswordHasher, Version,
        password_hash::{SaltString, rand_core::OsRng},
    },
    std::{convert::AsRef, error::Error},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Password(String);

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Password {
    pub fn parse(maybe_password: &str) -> Result<Self, String> {
        if maybe_password.trim().len() < 8 {
            return Err("Password should be longer than 8 characters".to_string());
        }

        Ok(Password(maybe_password.to_owned()))
    }

    pub fn hash(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(15000, 2, 1, None)?)
            .hash_password(self.as_ref().as_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }
}
