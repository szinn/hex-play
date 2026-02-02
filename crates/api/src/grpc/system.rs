use tonic::{Request, Response, Status};

use crate::grpc::{
    error::map_core_error,
    system_proto::{StatusRequest, StatusResponse, system_service_server::SystemService},
};

/// gRPC SystemService implementation
pub(crate) struct GrpcSystemService;

impl GrpcSystemService {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl SystemService for GrpcSystemService {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn status(&self, request: Request<StatusRequest>) -> Result<Response<StatusResponse>, Status> {
        let response = handler::status(request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }
}

pub(crate) mod handler {
    use hex_play_core::Error;

    use crate::grpc::system_proto::{StatusRequest, StatusResponse};

    pub(crate) async fn status(request: StatusRequest) -> Result<StatusResponse, Error> {
        Ok(StatusResponse {
            answer: format!("{}: Answered", request.question),
        })
    }
}

pub mod api {
    use hex_play_core::Error;

    use crate::grpc::system_proto::{StatusRequest, StatusResponse, system_service_client::SystemServiceClient};

    #[tracing::instrument(level = "trace")]
    pub async fn status(question: String) -> Result<String, Error> {
        let mut client = SystemServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::Any(Box::new(e)))?;

        let request = tonic::Request::new(StatusRequest { question });
        let response: StatusResponse = client.status(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();

        Ok(response.answer)
    }
}
