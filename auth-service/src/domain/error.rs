pub enum AuthAPIError {
    IncorrectCredentials,
    InvalidCredentials,
    InvalidToken,
    MissingToken,
    UserAlreadyExists,
    UnexpectedError,
}
