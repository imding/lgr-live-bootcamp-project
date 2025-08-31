use {
    crate::{
        app_state::AppState,
        domain::{email::Email, error::AuthAPIError, password::Password},
        utils::auth::generate_auth_cookie,
    },
    axum::{
        Json,
        extract::{State, rejection::JsonRejection},
        http::StatusCode,
        response::IntoResponse,
    },
    axum_extra::extract::CookieJar,
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
    jar: CookieJar,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    println!("/login");
    let Json(request) = match payload {
        Ok(request_json) => request_json,
        Err(rejection) => {
            let message = rejection.body_text();

            eprintln!("{message}");

            return Ok((jar, (rejection.status(), Json(LoginResponse { message }))));
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

    let Ok(auth_cookie) = generate_auth_cookie(&email)
    else {
        return Err(AuthAPIError::UnexpectedError);
    };
    let updated_jar = jar.add(auth_cookie);

    Ok((updated_jar, (StatusCode::OK, Json(LoginResponse { message: "User logged in successfully!".to_string() }))))
}
