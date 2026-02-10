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
    use std::sync::Arc;

    use axum::{
        Extension,
        http::{HeaderName, Request},
    };
    use hex_play_core::services::CoreServices;
    use tower::ServiceBuilder;
    use tower_http::{
        request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
        trace::TraceLayer,
    };

    use crate::HexPlayFrontend;

    const REQUEST_ID_HEADER: &str = "x-request-id";

    pub fn launch_server_frontend(core_services: Arc<CoreServices>) {
        std::thread::spawn(move || {
            tracing::info!("Frontend started");
            dioxus::serve(|| {
                let core_services = core_services.clone();
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
                        .layer(PropagateRequestIdLayer::new(x_request_id));

                    let router = dioxus::server::router(HexPlayFrontend).layer(Extension(core_services)).layer(middleware);
                    Ok(router)
                }
            })
        });
    }
}

#[component]
fn HexPlayFrontend() -> Element {
    rsx! {
        "Hello World"
    }
}
