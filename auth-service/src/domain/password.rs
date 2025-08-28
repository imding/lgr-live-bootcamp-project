use std::convert::AsRef;

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
}

