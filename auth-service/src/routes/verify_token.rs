use {
    crate::{app_state::AppState, domain::error::AuthAPIError, utils::auth::validate_token},
    axum::{Json, extract::State, http::StatusCode, response::IntoResponse},
    serde::{Deserialize, Serialize},
};

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct VerifyTokenResponse {
    message: String,
}

pub async fn verify_token(
    state: State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    println!("/verify-token");

    let parts = request.token.split('.').collect::<Vec<_>>();

    if parts.len() != 3 {
        return Err(AuthAPIError::MalformedToken);
    }

    if validate_token(Some(state.banned_token_store.clone()), &request.token).await.is_err() {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok((StatusCode::OK, Json(VerifyTokenResponse { message: "Token verified!".to_string() })))
}
