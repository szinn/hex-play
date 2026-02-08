use dioxus::prelude::*;

#[cfg(feature = "web")]
pub fn launch_web_frontend() {
    dioxus::launch(HexPlayFrontend)
}

#[cfg(feature = "server")]
pub fn launch_server_frontend() {
    dioxus::serve(|| async move {
        let router = dioxus::server::router(HexPlayFrontend);
        Ok(router)
    })
}

#[component]
fn HexPlayFrontend() -> Element {
    rsx! {
        "Hello World"
    }
}
