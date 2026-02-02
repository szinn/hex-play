use std::sync::Arc;

use hex_play_core::{Error, services::CoreServices};
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};
use tonic::{Request, Response, Status, transport::Server};

use crate::grpc::hex_play::{
    StatusRequest, StatusResponse,
    hex_play_server::{HexPlay, HexPlayServer},
};

pub mod system;

pub(crate) mod hex_play {
    tonic::include_proto!("hex_play");
}

pub(crate) struct GrpcSubsystem {
    // core_services: Arc<CoreServices>,
}

#[tonic::async_trait]
impl HexPlay for GrpcSubsystem {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn status(&self, request: Request<StatusRequest>) -> Result<Response<StatusResponse>, Status> {
        let response = system::handler::status(request.into_inner())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(response))
    }
}

impl GrpcSubsystem {
    pub(crate) fn new(_core_services: Arc<CoreServices>) -> Self {
        Self {}
    }
}

impl IntoSubsystem<Error> for GrpcSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
        let addr = "0.0.0.0:3001".parse().map_err(|_| Error::Message("Can't parse address".into()))?;

        tracing::info!("listening on {}", addr);
        tokio::select! {
            _ = subsys.on_shutdown_requested() => {
                tracing::info!("GrpcSubsystem shutting down...");
            }
            _ = Server::builder().add_service(HexPlayServer::new(self)).serve(addr) => {
                    subsys.request_shutdown();
                }
        }

        Ok(())
    }
}
