pub(crate) mod routes {
    use std::sync::Arc;

    use axum::{Router, extract::State, routing::put};
    use hex_play_core::services::CoreServices;

    use crate::http::error::Error;

    pub(crate) fn get_routes(core_services: Arc<CoreServices>) -> Router {
        Router::new().route("/api/user/v1/sample", put(sample_database_work)).with_state(core_services)
    }

    async fn sample_database_work(State(core_services): State<Arc<CoreServices>>) -> Result<(), Error> {
        core_services.user.sample_work().await.map_err(Error::Core)
    }
}
