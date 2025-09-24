use {
    argon2::{
        Algorithm, Argon2, Params, PasswordHasher, Version,
        password_hash::{SaltString, rand_core::OsRng},
    },
    color_eyre::{Result, eyre::eyre},
    secrecy::{ExposeSecret, SecretBox},
    std::convert::AsRef,
};

#[derive(Debug)]
pub struct Password(SecretBox<String>);

impl AsRef<SecretBox<String>> for Password {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Clone for Password {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl Password {
    pub fn parse(maybe_password: &SecretBox<String>) -> Result<Self> {
        if !validate_password(maybe_password) {
            return Err(eyre!("Password should be longer than 8 characters"));
        }

        Ok(Password(SecretBox::new(Box::new(maybe_password.expose_secret().to_owned()))))
    }

    pub fn hash(&self) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(15000, 2, 1, None)?)
            .hash_password(self.as_ref().expose_secret().as_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }
}

fn validate_password(secret: &SecretBox<String>) -> bool {
    secret.expose_secret().len() >= 8
}
