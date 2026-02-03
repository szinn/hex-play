use std::sync::Arc;

use hex_play_core::{Error, services::CoreServices};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};
use tonic::transport::Server;

use crate::error::ApiError;

mod error;
pub mod system;
pub mod user;

pub(crate) mod system_proto {
    tonic::include_proto!("hex_play.system");
}

pub(crate) mod user_proto {
    tonic::include_proto!("hex_play.user");
}

pub(crate) struct GrpcSubsystem {
    core_services: Arc<CoreServices>,
}

impl GrpcSubsystem {
    pub(crate) fn new(core_services: Arc<CoreServices>) -> Self {
        Self { core_services }
    }
}

impl IntoSubsystem<Error> for GrpcSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
        let addr = "0.0.0.0:3001".parse().map_err(|_| Error::from(ApiError::AddressParse("0.0.0.0:3001".into())))?;

        let system_service = system::GrpcSystemService::new();
        let user_service = user::GrpcUserService::new(self.core_services.clone());

        tracing::info!("listening on {}", addr);
        tokio::select! {
            _ = subsys.on_shutdown_requested() => {
                tracing::info!("GrpcSubsystem shutting down...");
            }
            _ = Server::builder()
                .add_service(system_proto::system_service_server::SystemServiceServer::new(system_service))
                .add_service(user_proto::user_service_server::UserServiceServer::new(user_service))
                .serve(addr) => {
                subsys.request_shutdown();
            }
        }

        Ok(())
    }
}
