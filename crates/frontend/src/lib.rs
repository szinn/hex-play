use dioxus::prelude::*;

pub(crate) mod user;

#[cfg(feature = "web")]
pub mod web {
    use crate::HexPlayFrontend;

    pub fn launch_web_frontend() {
        dioxus::launch(HexPlayFrontend)
    }
}

#[cfg(feature = "server")]
mod error;

#[cfg(feature = "server")]
pub use error::FrontendError;

#[cfg(feature = "server")]
pub mod server {
    use std::{collections::HashSet, fmt::Debug, sync::Arc};

    use axum::{
        Extension,
        http::{HeaderName, Request},
    };
    use axum_session::{DatabaseError, DatabasePool, SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::{AuthConfig, AuthSessionLayer, Authentication, HasPermission};
    use chrono::DateTime;
    use hex_play_core::{
        models::{session::NewSession, user::UserId},
        services::{CoreServices, SessionService},
    };
    use serde::{Deserialize, Serialize};
    use tower::ServiceBuilder;
    use tower_http::{
        request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
        trace::TraceLayer,
    };

    use crate::HexPlayFrontend;

    #[derive(Clone)]
    pub(crate) struct BackendSessionPool {
        session_service: Arc<dyn SessionService>,
    }

    impl Debug for BackendSessionPool {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BackendSessionPool").finish()
        }
    }

    pub(crate) type AuthSession = axum_session_auth::AuthSession<AuthUser, UserId, BackendSessionPool, BackendSessionPool>;

    #[async_trait::async_trait]
    impl DatabasePool for BackendSessionPool {
        #[tracing::instrument(level = "trace", skip(self))]
        async fn initiate(&self, _table_name: &str) -> Result<(), DatabaseError> {
            Ok(())
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn count(&self, _table_name: &str) -> Result<i64, DatabaseError> {
            self.session_service.count().await.map_err(|e| DatabaseError::GenericSelectError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn store(&self, id: &str, session: &str, expires: i64, _table_name: &str) -> Result<(), DatabaseError> {
            let expires_at = DateTime::from_timestamp(expires, 0).ok_or_else(|| DatabaseError::GenericInsertError(format!("invalid timestamp: {expires}")))?;
            let new_session = NewSession::new(id, session, expires_at).map_err(|e| DatabaseError::GenericInsertError(e.to_string()))?;
            self.session_service
                .store(new_session)
                .await
                .map_err(|e| DatabaseError::GenericInsertError(e.to_string()))?;
            Ok(())
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn load(&self, id: &str, _table_name: &str) -> Result<Option<String>, DatabaseError> {
            self.session_service
                .load(id)
                .await
                .map(|opt| opt.map(|s| s.session))
                .map_err(|e| DatabaseError::GenericSelectError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn delete_one_by_id(&self, id: &str, _table_name: &str) -> Result<(), DatabaseError> {
            self.session_service
                .delete_by_id(id)
                .await
                .map_err(|e| DatabaseError::GenericDeleteError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn exists(&self, id: &str, _table_name: &str) -> Result<bool, DatabaseError> {
            self.session_service
                .exists(id)
                .await
                .map_err(|e| DatabaseError::GenericSelectError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn delete_by_expiry(&self, _table_name: &str) -> Result<Vec<String>, DatabaseError> {
            self.session_service
                .delete_by_expiry()
                .await
                .map_err(|e| DatabaseError::GenericDeleteError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn delete_all(&self, _table_name: &str) -> Result<(), DatabaseError> {
            self.session_service
                .delete_all()
                .await
                .map_err(|e| DatabaseError::GenericDeleteError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn get_ids(&self, _table_name: &str) -> Result<Vec<String>, DatabaseError> {
            self.session_service
                .get_ids()
                .await
                .map_err(|e| DatabaseError::GenericSelectError(e.to_string()))
        }

        #[tracing::instrument(level = "trace", skip(self))]
        fn auto_handles_expiry(&self) -> bool {
            false
        }
    }

    /// Stub user type for session authentication.
    /// TODO: Replace with a real user backed by CoreServices.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub(crate) struct AuthUser {
        id: UserId,
        anonymous: bool,
        pub username: String,
        pub permissions: HashSet<String>,
    }

    impl Default for AuthUser {
        fn default() -> Self {
            Self {
                id: 1,
                anonymous: true,
                username: "1".into(),
                permissions: HashSet::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Authentication<Self, UserId, BackendSessionPool> for AuthUser {
        #[tracing::instrument(level = "trace", skip(_pool))]
        async fn load_user(userid: UserId, _pool: Option<&BackendSessionPool>) -> Result<Self, anyhow::Error> {
            let mut permissions = HashSet::new();
            if userid == 2 {
                permissions.insert("Admin::View".into());
            }
            Ok(Self {
                id: userid,
                anonymous: userid == 1,
                username: format!("{userid}"),
                permissions,
            })
        }

        #[tracing::instrument(level = "trace")]
        fn is_authenticated(&self) -> bool {
            !self.anonymous
        }

        #[tracing::instrument(level = "trace")]
        fn is_active(&self) -> bool {
            !self.anonymous
        }

        #[tracing::instrument(level = "trace")]
        fn is_anonymous(&self) -> bool {
            self.anonymous
        }
    }

    #[async_trait::async_trait]
    impl HasPermission<BackendSessionPool> for AuthUser {
        #[tracing::instrument(level = "trace", skip(self, _pool))]
        async fn has(&self, perm: &str, _pool: &Option<&BackendSessionPool>) -> bool {
            self.permissions.contains(perm)
        }
    }

    const REQUEST_ID_HEADER: &str = "x-request-id";

    pub fn launch_server_frontend(core_services: Arc<CoreServices>) {
        std::thread::spawn(move || {
            tracing::info!("Frontend started");
            dioxus::serve(|| {
                let core_services = core_services.clone();
                let backend_pool = BackendSessionPool {
                    session_service: core_services.session_service.clone(),
                };
                let session_config = SessionConfig::default();
                let auth_config = AuthConfig::<UserId>::default().with_anonymous_user_id(Some(1));
                async move {
                    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

                    let middleware = ServiceBuilder::new()
                        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                            let request_id = request
                                .headers()
                                .get(REQUEST_ID_HEADER)
                                .map(|v| v.to_str().unwrap_or_default())
                                .unwrap_or_default();

                            tracing::info_span!(
                                "http",
                                request_id = ?request_id,
                            )
                        }))
                        .layer(PropagateRequestIdLayer::new(x_request_id))
                        .layer(SessionLayer::new(
                            SessionStore::<BackendSessionPool>::new(Some(backend_pool.clone()), session_config).await?,
                        ))
                        .layer(AuthSessionLayer::<AuthUser, UserId, BackendSessionPool, BackendSessionPool>::new(Some(backend_pool)).with_config(auth_config));

                    let router = dioxus::server::router(HexPlayFrontend).layer(Extension(core_services)).layer(middleware);
                    Ok(router)
                }
            })
        });
    }
}

use user::{get_permissions, get_user_name, login, logout};

#[component]
fn HexPlayFrontend() -> Element {
    let mut login = use_action(login);
    let mut user_name = use_action(get_user_name);
    let mut permissions = use_action(get_permissions);
    let mut logout = use_action(logout);

    let fetch_new = move |_| async move {
        user_name.call().await;
        permissions.call().await;
    };

    rsx! {
        div {
            button {
                onclick: move |_| async move {
                    login.call().await;
                },
                "Login Test User"
            }
            button {
                onclick: move |_| async move {
                    logout.call().await;
                },
                "Logout"
            }
            button {
                onclick: fetch_new,
                "Fetch User Info"
            }

            pre { "Logged in: {login.value():?}" }
            pre { "User name: {user_name.value():?}" }
            pre { "Permissions: {permissions.value():?}" }
        }
    }
}
