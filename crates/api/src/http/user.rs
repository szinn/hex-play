pub(crate) mod routes {
    use std::sync::Arc;

    use axum::{
        Json, Router,
        extract::{Path, Query, State},
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
    async fn create_user(State(core_services): State<Arc<CoreServices>>, Json(request): Json<CreateUserRequest>) -> Result<Json<UserResponse>, Error> {
        let new_user = NewUser {
            name: request.name,
            email: request.email,
        };
        let user = core_services.user.add_user(new_user).await.map_err(Error::Core)?;

        Ok(Json(user.into()))
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
            .map(|user| user.into())
            .collect();

        let response = ListUsersResponse { users };

        Ok(Json(response))
    }

    #[tracing::instrument(level = "trace", skip(core_services))]
    async fn get_user(Path(id): Path<i64>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
        let user = core_services.user.find_by_id(id).await.map_err(Error::Core)?;

        if let Some(user) = user {
            return Ok(Json(user.into()));
        }

        Err(Error::NotFound)
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UpdateUserRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    }

    #[tracing::instrument(level = "trace", skip(core_services))]
    async fn update_user(
        Path(id): Path<i64>,
        State(core_services): State<Arc<CoreServices>>,
        Json(request): Json<UpdateUserRequest>,
    ) -> Result<Json<UserResponse>, Error> {
        let user = core_services.user.find_by_id(id).await.map_err(Error::Core)?;

        if let Some(mut user) = user {
            if let Some(name) = request.name {
                user.name = name;
            }
            if let Some(email) = request.email {
                user.email = email;
            }

            let user = core_services.user.update_user(user).await.map_err(Error::Core)?;

            return Ok(Json(user.into()));
        }

        Err(Error::NotFound)
    }

    #[tracing::instrument(level = "trace", skip(core_services))]
    async fn delete_user(Path(id): Path<i64>, State(core_services): State<Arc<CoreServices>>) -> Result<Json<UserResponse>, Error> {
        let user = core_services.user.delete_user(id).await.map_err(Error::Core)?;

        return Ok(Json(user.into()));
    }
}
