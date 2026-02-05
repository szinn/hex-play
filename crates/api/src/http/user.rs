use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use hex_play_core::{
    models::{Age, Email, NewUser, PartialUserUpdate, User},
    services::CoreServices,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::error::Error;

pub(crate) fn get_routes(core_services: Arc<CoreServices>) -> Router {
    Router::new()
        .nest(
            "/api/v1/user",
            Router::new()
                .route("/", post(create_user).get(list_users))
                .route("/token/{token}", get(get_user_by_token))
                .route("/{id}", get(get_user).patch(update_user).delete(delete_user)),
        )
        .with_state(core_services)
}

#[derive(Deserialize, Debug)]
struct CreateUserRequest {
    name: String,
    email: Email,
    #[serde(default)]
    age: Age,
}

impl From<CreateUserRequest> for NewUser {
    fn from(req: CreateUserRequest) -> Self {
        Self {
            name: req.name,
            email: req.email,
            age: req.age,
        }
    }
}

#[derive(Serialize, Debug)]
struct UserResponse {
    id: i64,
    token: Uuid,
    name: String,
    email: Email,
    age: Age,
    version: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            token: user.token,
            name: user.name,
            email: user.email,
            age: user.age,
            version: user.version,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn create_user(
    State(core_services): State<Arc<CoreServices>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), Error> {
    let user = core_services.user_service.add_user(request.into()).await.map_err(Error::Core)?;
    Ok((StatusCode::CREATED, Json(user.into())))
}

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub start_id: Option<i64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct ListUsersResponse {
    users: Vec<UserResponse>,
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn list_users(Query(opts): Query<FilterOptions>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<ListUsersResponse>, Error> {
    let users = core_services
        .user_service
        .list_users(opts.start_id, opts.page_size)
        .await
        .map_err(Error::Core)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ListUsersResponse { users }))
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn get_user(Path(id): Path<i64>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
    let user = core_services.user_service.find_by_id(id).await.map_err(Error::Core)?.ok_or(Error::NotFound)?;
    Ok(Json(user.into()))
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn get_user_by_token(Path(token): Path<Uuid>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
    let user = core_services
        .user_service
        .find_by_token(token)
        .await
        .map_err(Error::Core)?
        .ok_or(Error::NotFound)?;
    Ok(Json(user.into()))
}

#[derive(Deserialize, Debug)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<Email>,
    age: Option<Age>,
}

impl From<UpdateUserRequest> for PartialUserUpdate {
    fn from(req: UpdateUserRequest) -> Self {
        Self {
            name: req.name,
            email: req.email,
            age: req.age,
        }
    }
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn update_user(
    Path(id): Path<i64>,
    State(core_services): State<Arc<CoreServices>>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, Error> {
    let mut user = core_services.user_service.find_by_id(id).await.map_err(Error::Core)?.ok_or(Error::NotFound)?;

    let update: PartialUserUpdate = request.into();
    update.apply_to(&mut user);

    let user = core_services.user_service.update_user(user).await.map_err(Error::Core)?;
    Ok(Json(user.into()))
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn delete_user(Path(id): Path<i64>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
    let user = core_services.user_service.delete_user(id).await.map_err(Error::Core)?;
    Ok(Json(user.into()))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use hex_play_core::{
        Error, RepositoryError,
        models::User,
        test_support::{MockUserService, create_arc_core_services_with_mock},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::get_routes;

    // ===================
    // Test Helpers
    // ===================
    fn create_test_app(mock: MockUserService) -> Router {
        get_routes(create_arc_core_services_with_mock(mock))
    }

    async fn body_to_string(body: Body) -> String {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.expect("failed to read response body");
        String::from_utf8(bytes.to_vec()).expect("response body must be valid UTF-8")
    }

    // ===================
    // Tests: POST /api/v1/user (create_user)
    // ===================
    #[tokio::test]
    async fn test_create_user_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_add_user_result(Ok(user));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/user")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Doe","email":"john@example.com","age":30}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""email":"john@example.com""#));
        assert!(body.contains(r#""age":30"#));
        assert!(body.contains(r#""version":0"#));
        assert!(body.contains(r#""created_at":"#));
        assert!(body.contains(r#""updated_at":"#));
    }

    #[tokio::test]
    async fn test_create_user_without_age_defaults_to_zero() {
        let user = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserService::default().with_add_user_result(Ok(user));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/user")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Doe","email":"john@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""age":0"#));
    }

    #[tokio::test]
    async fn test_create_user_invalid_json() {
        let mock = MockUserService::default();
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/user")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Doe""#)) // Invalid JSON
                    .unwrap(),
            )
            .await
            .unwrap();

        // Axum returns 400 for malformed JSON
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_user_missing_fields() {
        let mock = MockUserService::default();
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/user")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Doe"}"#)) // Missing email
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_user_constraint_violation() {
        let mock = MockUserService::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate email".into()))));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/user")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Doe","email":"john@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ===================
    // Tests: GET /api/v1/user (list_users)
    // ===================
    #[tokio::test]
    async fn test_list_users_success() {
        let users = vec![
            User::test_with_age(1, "John Doe", "john@example.com", 30),
            User::test_with_age(2, "Jane Doe", "jane@example.com", 25),
        ];
        let mock = MockUserService::default().with_list_users_result(Ok(users));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""users""#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""name":"Jane Doe""#));
        assert!(body.contains(r#""age":30"#));
        assert!(body.contains(r#""age":25"#));
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let mock = MockUserService::default().with_list_users_result(Ok(vec![]));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""users":[]"#));
    }

    #[tokio::test]
    async fn test_list_users_with_pagination() {
        let users = vec![User::test(5, "User Five", "five@example.com")];
        let mock = MockUserService::default().with_list_users_result(Ok(users));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/user?start_id=5&page_size=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_users_invalid_start_id() {
        let mock = MockUserService::default().with_list_users_result(Err(Error::InvalidId(-1)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user?start_id=-1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_users_invalid_page_size() {
        let mock = MockUserService::default().with_list_users_result(Err(Error::InvalidPageSize(0)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user?page_size=0").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ===================
    // Tests: GET /api/v1/user/{id} (get_user)
    // ===================
    #[tokio::test]
    async fn test_get_user_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_find_by_id_result(Ok(Some(user)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user/1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""age":30"#));
        assert!(body.contains(r#""version":0"#));
        assert!(body.contains(r#""created_at":"#));
        assert!(body.contains(r#""updated_at":"#));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mock = MockUserService::default().with_find_by_id_result(Ok(None));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user/999").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_user_invalid_id() {
        let mock = MockUserService::default().with_find_by_id_result(Err(Error::InvalidId(-1)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user/-1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ===================
    // Tests: PATCH /api/v1/user/{id} (update_user)
    // ===================
    #[tokio::test]
    async fn test_update_user_success() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let updated = User::test_with_age(1, "John Updated", "john@example.com", 30);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/user/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"John Updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""name":"John Updated""#));
        assert!(body.contains(r#""age":30"#));
    }

    #[tokio::test]
    async fn test_update_user_partial_email() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 25);
        let updated = User::test_with_age(1, "John Doe", "john.new@example.com", 25);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/user/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"email":"john.new@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""email":"john.new@example.com""#));
        assert!(body.contains(r#""age":25"#));
    }

    #[tokio::test]
    async fn test_update_user_age() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let updated = User::test_with_age(1, "John Doe", "john@example.com", 31);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/user/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"age":31}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""age":31"#));
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mock = MockUserService::default().with_find_by_id_result(Ok(None));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/user/999")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_user_conflict() {
        let existing = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Err(Error::RepositoryError(RepositoryError::Conflict)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/user/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    // ===================
    // Tests: DELETE /api/v1/user/{id} (delete_user)
    // ===================
    #[tokio::test]
    async fn test_delete_user_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_delete_user_result(Ok(user));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("DELETE").uri("/api/v1/user/1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""age":30"#));
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mock = MockUserService::default().with_delete_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("DELETE").uri("/api/v1/user/999").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ===================
    // Tests: GET /api/v1/user/token/{token} (get_user_by_token)
    // ===================
    #[tokio::test]
    async fn test_get_user_by_token_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let token = user.token;
        let mock = MockUserService::default().with_find_by_token_result(Ok(Some(user)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/user/token/{token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(&format!(r#""token":"{token}""#)));
        assert!(body.contains(r#""age":30"#));
    }

    #[tokio::test]
    async fn test_get_user_by_token_not_found() {
        let mock = MockUserService::default().with_find_by_token_result(Ok(None));
        let app = create_test_app(mock);

        let token = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/user/token/{token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_user_by_token_invalid_uuid() {
        let mock = MockUserService::default();
        let app = create_test_app(mock);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/user/token/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
