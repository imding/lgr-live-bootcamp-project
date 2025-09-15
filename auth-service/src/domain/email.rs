use {
    serde::{Deserialize, Deserializer},
    std::convert::AsRef,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> Deserialize<'a> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let maybe_email = String::deserialize(deserializer)?;
        Email::parse(&maybe_email).map_err(serde::de::Error::custom)
    }
}

impl Email {
    pub fn parse(maybe_email: &str) -> Result<Self, String> {
        if !maybe_email.trim().contains("@") {
            return Err("Email should contain @".to_string());
        }

        Ok(Email(maybe_email.to_owned()))
    }
}
