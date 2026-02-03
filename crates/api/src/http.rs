use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderName, Request},
    response::Html,
    routing::get,
};
use hex_play_core::{Error, services::CoreServices};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::error::ApiError;

mod error;
mod user;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub(crate) struct HttpSubsystem {
    core_services: Arc<CoreServices>,
}

impl HttpSubsystem {
    pub(crate) fn new(core_services: Arc<CoreServices>) -> Self {
        Self { core_services }
    }
}

impl IntoSubsystem<Error> for HttpSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
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
            .layer(PropagateRequestIdLayer::new(x_request_id));

        let user_routes = user::get_routes(self.core_services.clone());
        let app = Router::new().route("/", get(hello_handler)).merge(user_routes).layer(middleware);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .map_err(|e| Error::from(ApiError::Network(e.to_string())))?;
        tracing::info!(
            "listening on {}",
            listener.local_addr().map_err(|e| Error::from(ApiError::Network(e.to_string())))?
        );

        tokio::select! {
            _ = subsys.on_shutdown_requested() => {
                tracing::info!("HttpSubsystem shutting down...");
            }
            result = axum::serve(listener, app) => {
                if let Err(e) = result {
                    tracing::error!("HTTP server error: {}", e);
                }
                subsys.request_shutdown();
            }
        }

        Ok(())
    }
}

async fn hello_handler() -> Html<&'static str> {
    tracing::info!("Hello world!");
    Html("<h1>Hello, World!</h1>")
}
