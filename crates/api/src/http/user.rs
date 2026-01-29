pub(crate) mod routes {
    use std::sync::Arc;

    use axum::{
        Json, Router,
        extract::{Path, Query, State},
        routing::{get, post},
    };
    use hex_play_core::services::CoreServices;
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};

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

    #[derive(Serialize, Deserialize, Debug)]
    struct CreateUserRequest {
        name: String,
        email: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UserResponse {
        id: i64,
        name: String,
        email: String,
    }

    #[tracing::instrument(level = "trace", skip(_core_services))]
    async fn create_user(State(_core_services): State<Arc<CoreServices>>, Json(_request): Json<CreateUserRequest>) -> Result<Json<UserResponse>, StatusCode> {
        let response = UserResponse {
            id: 10,
            name: "Fred".into(),
            email: "fred@wombat.com".into(),
        };

        Ok(Json(response))
    }

    #[derive(Deserialize, Debug, Default)]
    pub struct FilterOptions {
        pub start_id: Option<usize>,
        pub limit: Option<usize>,
    }
    #[tracing::instrument(level = "trace", skip(_core_services))]
    async fn list_users(Query(_opts): Query<FilterOptions>, State(_core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, StatusCode> {
        let response = UserResponse {
            id: 10,
            name: "Fred".into(),
            email: "fred@wombat.com".into(),
        };

        Ok(Json(response))
    }

    #[tracing::instrument(level = "trace", skip(_core_services))]
    async fn get_user(Path(_id): Path<i64>, State(_core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, StatusCode> {
        let response = UserResponse {
            id: 10,
            name: "Fred".into(),
            email: "fred@wombat.com".into(),
        };

        Ok(Json(response))
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UpdateUserRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    }

    #[tracing::instrument(level = "trace", skip(_core_services))]
    async fn update_user(
        Path(_id): Path<i64>,
        State(_core_services): State<Arc<CoreServices>>,
        Json(_request): Json<UpdateUserRequest>,
    ) -> Result<Json<UserResponse>, StatusCode> {
        let response = UserResponse {
            id: 10,
            name: "Fred".into(),
            email: "fred@wombat.com".into(),
        };

        Ok(Json(response))
    }

    #[tracing::instrument(level = "trace", skip(_core_services))]
    async fn delete_user(Path(_id): Path<i64>, State(_core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, StatusCode> {
        let response = UserResponse {
            id: 10,
            name: "Fred".into(),
            email: "fred@wombat.com".into(),
        };

        Ok(Json(response))
    }
}
