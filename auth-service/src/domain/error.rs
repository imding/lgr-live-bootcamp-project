use {color_eyre::eyre::Report, thiserror::Error};

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Malformed token")]
    MalformedToken,
    #[error("Missing token")]
    MissingToken,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}
