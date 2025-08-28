use std::convert::AsRef;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
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
