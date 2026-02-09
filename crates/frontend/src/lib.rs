use dioxus::prelude::*;

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

    use hex_play_core::services::CoreServices;

    use crate::HexPlayFrontend;

    pub fn launch_server_frontend(_core_services: Arc<CoreServices>) {
        std::thread::spawn(|| {
            tracing::info!("Frontend started");
            dioxus::serve(|| async move {
                let router = dioxus::server::router(HexPlayFrontend);
                Ok(router)
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
