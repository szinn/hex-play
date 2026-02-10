use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {
    hex_play_core::{models::User, services::CoreServices},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Debug)]
struct UserResponse {
    id: i64,
    token: String,
    name: String,
    email: String,
    age: i64,
    version: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListUsersResponse {
    users: Vec<UserResponse>,
}

#[cfg(feature = "server")]
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            token: user.token.to_string(),
            name: user.name,
            email: user.email.into_inner(),
            age: i64::from(user.age.value()),
            version: user.version,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[get("/api/v1/user", core_services: axum::Extension<Arc<CoreServices>>)]
#[tracing::instrument(level = "trace", skip(core_services))]
async fn get_users() -> Result<ListUsersResponse, ServerFnError> {
    let users = core_services
        .user_service
        .list_users(None, None)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
    let response = ListUsersResponse { users };
    Ok(response)
}
