use {
    color_eyre::eyre::{Result, eyre},
    secrecy::{ExposeSecret, SecretBox},
    serde::{Deserialize, Deserializer},
    std::convert::AsRef,
};

#[derive(Debug)]
pub struct Email(SecretBox<String>);

impl AsRef<SecretBox<String>> for Email {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Clone for Email {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl<'a> Deserialize<'a> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let maybe_email = String::deserialize(deserializer)?;
        let secret = SecretBox::new(Box::new(maybe_email));

        Email::parse(&secret).map_err(serde::de::Error::custom)
    }
}

impl Email {
    pub fn parse(maybe_email: &SecretBox<String>) -> Result<Self> {
        if !validate_email(maybe_email) {
            return Err(eyre!("Email should contain @"));
        }

        Ok(Email(SecretBox::new(Box::new(maybe_email.expose_secret().to_owned()))))
    }
}

fn validate_email(secret: &SecretBox<String>) -> bool {
    secret.expose_secret().contains("@")
}
