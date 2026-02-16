use std::collections::HashSet;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {
    crate::server::AuthSession,
    hex_play_core::{models::User, services::CoreServices},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Debug)]
struct UserResponse {
    id: u64,
    token: String,
    name: String,
    email: String,
    age: i64,
    version: u64,
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

/// We use the `auth::Session` extractor to get access to the current user session.
/// This lets us modify the user session, log in/out, and access the current user.
#[post("/api/user/login", auth: axum::Extension<AuthSession>)]
#[tracing::instrument(level = "trace", skip(auth))]
pub async fn login() -> Result<()> {
    auth.login_user(2);
    Ok(())
}

/// Just like `login`, but this time we log out the user.
#[post("/api/user/logout", auth: axum::Extension<AuthSession>)]
#[tracing::instrument(level = "trace", skip(auth))]
pub async fn logout() -> Result<()> {
    auth.logout_user();
    Ok(())
}

/// We can access the current user via `auth.current_user`.
/// We can have both anonymous user (id 1) and a logged in user (id 2).
///
/// Logged-in users will have more permissions which we can modify.
#[post("/api/user/name", auth: axum::Extension<AuthSession>)]
#[tracing::instrument(level = "trace", skip(auth))]
pub async fn get_user_name() -> Result<String> {
    let current_user = auth.current_user.clone();
    tracing::info!("Current user={:?}", current_user);
    Ok(current_user.unwrap().username)
}

/// Get the current user's permissions, guarding the endpoint with the `Auth` validator.
/// If this returns false, we use the `or_unauthorized` extension to return a 401 error.
#[get("/api/user/permissions", auth: axum::Extension<AuthSession>)]
#[tracing::instrument(level = "trace", skip(auth))]
pub async fn get_permissions() -> Result<HashSet<String>> {
    use axum_session_auth::{Auth, Rights};
    use hex_play_core::models::user::UserId;

    use crate::server::{AuthUser, BackendSessionPool};

    let user = auth.current_user.clone().unwrap();

    Auth::<AuthUser, UserId, BackendSessionPool>::build([axum::http::Method::GET], false)
        .requires(Rights::any([Rights::permission("Category::View"), Rights::permission("Admin::View")]))
        .validate(&user, &axum::http::Method::GET, None)
        .await
        .or_unauthorized("You do not have permission to view categories")?;

    Ok(user.permissions)
}
