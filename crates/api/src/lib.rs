use std::sync::Arc;

use hex_play_core::{Error, services::CoreServices};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemBuilder, SubsystemHandle};

use crate::{grpc::GrpcSubsystem, http::HttpSubsystem};

mod error;
pub mod grpc;
mod http;

pub use error::ApiError;

pub struct ApiSubsystem {
    core_services: Arc<CoreServices>,
}

impl IntoSubsystem<Error> for ApiSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
        tracing::info!("ApiSubsystem starting...");
        let http_subsystem = HttpSubsystem::new(self.core_services.clone());
        let grpc_subsystem = GrpcSubsystem::new(self.core_services.clone());

        subsys.start(SubsystemBuilder::new("Http", http_subsystem.into_subsystem()));
        subsys.start(SubsystemBuilder::new("Grpc", grpc_subsystem.into_subsystem()));

        tracing::info!("ApiSubsystem started");

        subsys.on_shutdown_requested().await;
        tracing::info!("ApiSubsystem shutting down");

        Ok(())
    }
}

pub fn create_api_subsystem(core_services: Arc<CoreServices>) -> ApiSubsystem {
    ApiSubsystem { core_services }
}
