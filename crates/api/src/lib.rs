use std::sync::Arc;

use hex_play_core::{CoreServices, Error};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemBuilder, SubsystemHandle};

use crate::http::HttpSubsystem;

mod http;

pub struct ApiSubsystem {
    core_services: Arc<CoreServices>,
}

impl IntoSubsystem<Error> for ApiSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
        tracing::info!("ApiSubsystem starting...");
        let http_subsystem = HttpSubsystem::new(self.core_services.clone());

        subsys.start(SubsystemBuilder::new("Http", http_subsystem.into_subsystem()));

        tracing::info!("ApiSubsystem started");
        subsys.on_shutdown_requested().await;
        tracing::info!("ApiSubsystem shutting down");
        Ok(())
    }
}

pub fn create_api_subsystem(core_services: Arc<CoreServices>) -> ApiSubsystem {
    ApiSubsystem { core_services }
}
