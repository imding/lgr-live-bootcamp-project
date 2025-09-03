pub enum AuthAPIError {
    IncorrectCredentials,
    InvalidCredentials,
    InvalidToken,
    MalformedToken,
    MissingToken,
    UserAlreadyExists,
    UnexpectedError,
}
