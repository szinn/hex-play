use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use hex_play_core::{
    models::{NewUser, User},
    services::CoreServices,
};
use serde::{Deserialize, Serialize};

use crate::http::error::Error;

pub(crate) fn get_routes(core_services: Arc<CoreServices>) -> Router {
    Router::new()
        .nest(
            "/api/v1/user",
            Router::new()
                .route("/", post(create_user).get(list_users))
                .route("/{id}", get(get_user).patch(update_user).delete(delete_user)),
        )
        .with_state(core_services)
}

#[derive(Deserialize, Debug)]
struct CreateUserRequest {
    name: String,
    email: String,
}

impl From<CreateUserRequest> for NewUser {
    fn from(req: CreateUserRequest) -> Self {
        Self {
            name: req.name,
            email: req.email,
        }
    }
}

#[derive(Serialize, Debug)]
struct UserResponse {
    id: i64,
    name: String,
    email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
        }
    }
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn create_user(
    State(core_services): State<Arc<CoreServices>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), Error> {
    let user = core_services.user.add_user(request.into()).await.map_err(Error::Core)?;
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
        .user
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
    let user = core_services.user.find_by_id(id).await.map_err(Error::Core)?.ok_or(Error::NotFound)?;
    Ok(Json(user.into()))
}

#[derive(Deserialize, Debug)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn update_user(
    Path(id): Path<i64>,
    State(core_services): State<Arc<CoreServices>>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, Error> {
    let mut user = core_services.user.find_by_id(id).await.map_err(Error::Core)?.ok_or(Error::NotFound)?;

    if let Some(name) = request.name {
        user.name = name;
    }
    if let Some(email) = request.email {
        user.email = email;
    }

    let user = core_services.user.update_user(user).await.map_err(Error::Core)?;
    Ok(Json(user.into()))
}

#[tracing::instrument(level = "trace", skip(core_services))]
async fn delete_user(Path(id): Path<i64>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
    let user = core_services.user.delete_user(id).await.map_err(Error::Core)?;
    Ok(Json(user.into()))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use hex_play_core::{
        Error, RepositoryError,
        models::{NewUser, User},
        services::CoreServices,
        use_cases::UserUseCases,
    };
    use tower::ServiceExt;

    use super::get_routes;

    // ===================
    // Mock UserUseCases
    // ===================
    #[derive(Default)]
    struct MockUserUseCases {
        add_user_result: Mutex<Option<Result<User, Error>>>,
        update_user_result: Mutex<Option<Result<User, Error>>>,
        delete_user_result: Mutex<Option<Result<User, Error>>>,
        find_by_id_result: Mutex<Option<Result<Option<User>, Error>>>,
        list_users_result: Mutex<Option<Result<Vec<User>, Error>>>,
    }

    impl MockUserUseCases {
        fn with_add_user_result(self, result: Result<User, Error>) -> Self {
            *self.add_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_update_user_result(self, result: Result<User, Error>) -> Self {
            *self.update_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_user_result(self, result: Result<User, Error>) -> Self {
            *self.delete_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_id_result(self, result: Result<Option<User>, Error>) -> Self {
            *self.find_by_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_list_users_result(self, result: Result<Vec<User>, Error>) -> Self {
            *self.list_users_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait::async_trait]
    impl UserUseCases for MockUserUseCases {
        async fn add_user(&self, _user: NewUser) -> Result<User, Error> {
            self.add_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn update_user(&self, _user: User) -> Result<User, Error> {
            self.update_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn list_users(&self, _start_id: Option<i64>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
            self.list_users_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn delete_user(&self, _id: i64) -> Result<User, Error> {
            self.delete_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn find_by_id(&self, _id: i64) -> Result<Option<User>, Error> {
            self.find_by_id_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }
    }

    // ===================
    // Test Helpers
    // ===================
    fn create_test_app(mock: MockUserUseCases) -> Router {
        let core_services = Arc::new(CoreServices { user: Arc::new(mock) });
        get_routes(core_services)
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
        let user = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserUseCases::default().with_add_user_result(Ok(user));
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
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
        assert!(body.contains(r#""email":"john@example.com""#));
    }

    #[tokio::test]
    async fn test_create_user_invalid_json() {
        let mock = MockUserUseCases::default();
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
        let mock = MockUserUseCases::default();
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
        let mock = MockUserUseCases::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate email".into()))));
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
        let users = vec![User::test(1, "John Doe", "john@example.com"), User::test(2, "Jane Doe", "jane@example.com")];
        let mock = MockUserUseCases::default().with_list_users_result(Ok(users));
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
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let mock = MockUserUseCases::default().with_list_users_result(Ok(vec![]));
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
        let mock = MockUserUseCases::default().with_list_users_result(Ok(users));
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
        let mock = MockUserUseCases::default().with_list_users_result(Err(Error::InvalidId(-1)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user?start_id=-1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_users_invalid_page_size() {
        let mock = MockUserUseCases::default().with_list_users_result(Err(Error::InvalidPageSize(0)));
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
        let user = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserUseCases::default().with_find_by_id_result(Ok(Some(user)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user/1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mock = MockUserUseCases::default().with_find_by_id_result(Ok(None));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("GET").uri("/api/v1/user/999").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_user_invalid_id() {
        let mock = MockUserUseCases::default().with_find_by_id_result(Err(Error::InvalidId(-1)));
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
        let existing = User::test(1, "John Doe", "john@example.com");
        let updated = User::test(1, "John Updated", "john@example.com");
        let mock = MockUserUseCases::default()
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
    }

    #[tokio::test]
    async fn test_update_user_partial_email() {
        let existing = User::test(1, "John Doe", "john@example.com");
        let updated = User::test(1, "John Doe", "john.new@example.com");
        let mock = MockUserUseCases::default()
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
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mock = MockUserUseCases::default().with_find_by_id_result(Ok(None));
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
        let mock = MockUserUseCases::default()
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
        let user = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserUseCases::default().with_delete_user_result(Ok(user));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("DELETE").uri("/api/v1/user/1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains(r#""id":1"#));
        assert!(body.contains(r#""name":"John Doe""#));
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mock = MockUserUseCases::default().with_delete_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let app = create_test_app(mock);

        let response = app
            .oneshot(Request::builder().method("DELETE").uri("/api/v1/user/999").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
