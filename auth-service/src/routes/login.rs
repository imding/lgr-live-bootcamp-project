use {
    crate::{
        app_state::AppState,
        domain::{email::Email, error::AuthAPIError, password::Password},
    },
    axum::{
        Json,
        extract::{State, rejection::JsonRejection},
        http::StatusCode,
        response::IntoResponse,
    },
    serde::{Deserialize, Serialize},
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LoginResponse {
    pub message: String,
}

pub async fn login(
    state: State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AuthAPIError> {
    println!("/login");
    let Json(request) = match payload {
        Ok(request_json) => request_json,
        Err(rejection) => {
            let message = rejection.body_text();

            eprintln!("{message}");

            return Ok((rejection.status(), Json(LoginResponse { message })));
        }
    };
    let Ok(email) = Email::parse(&request.email)
    else {
        return Err(AuthAPIError::InvalidCredentials);
    };
    let (user, password) = match (state.user_store.get_user(&email).await, Password::parse(&request.password)) {
        (Ok(user), Ok(password)) => (user, password),
        _ => return Err(AuthAPIError::InvalidCredentials),
    };

    if user.password != password {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    Ok((StatusCode::OK, Json(LoginResponse { message: "User logged in successfully!".to_string() })))
}
