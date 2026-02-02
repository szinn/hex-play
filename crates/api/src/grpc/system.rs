pub(crate) mod handler {
    use hex_play_core::Error;

    use crate::grpc::hex_play::{StatusRequest, StatusResponse};

    pub(crate) async fn status(request: StatusRequest) -> Result<StatusResponse, Error> {
        Ok(StatusResponse {
            answer: format!("{}: Answered", request.question),
        })
    }
}

pub mod api {
    use hex_play_core::Error;

    use crate::grpc::hex_play::{StatusRequest, StatusResponse, hex_play_client::HexPlayClient};

    #[tracing::instrument(level = "trace")]
    pub async fn status(question: String) -> Result<String, Error> {
        let mut client = HexPlayClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;

        let request = tonic::Request::new(StatusRequest { question });
        let response: StatusResponse = client.status(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();

        Ok(response.answer)
    }
}
