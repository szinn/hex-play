use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::{HeaderName, Request},
    response::Html,
    routing::get,
};
use hex_play_core::{CoreServices, Error};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};
use tokio_util::sync::CancellationToken;
use tower::{Service, ServiceBuilder};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub struct HttpSubsystem {
    core_services: Arc<CoreServices>,
}

impl HttpSubsystem {
    pub(crate) fn new(core_services: Arc<CoreServices>) -> Self {
        Self { core_services }
    }
}

impl IntoSubsystem<Error> for HttpSubsystem {
    async fn run(self, subsys: &mut SubsystemHandle) -> Result<(), Error> {
        let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

        let middleware = ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid))
            .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the request id as generated.
                let request_id = request.headers().get(REQUEST_ID_HEADER);

                match request_id {
                    Some(request_id) => tracing::info_span!(
                        "http_request",
                        request_id = ?request_id,
                    ),
                    None => {
                        tracing::error!("could not extract request_id");
                        tracing::info_span!("http_request")
                    }
                }
            }))
            // send headers from request to response headers
            .layer(PropagateRequestIdLayer::new(x_request_id));

        // build our application with a route
        let app = Router::new().route("/", get(hello_handler)).layer(middleware);

        // run it
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        println!("listening on {}", listener.local_addr().unwrap());

        let cancel_token = CancellationToken::new();

        loop {
            let (socket, remote_addr) = tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    break;
                }

                result = listener.accept() => {
                    result.unwrap()
                }
            };
            tracing::debug!("connection {} accepted", remote_addr);
            let tower_service = app.clone();
            let token = cancel_token.clone();
            tokio::spawn(async move {
                if let Err(e) = handler(socket, remote_addr, tower_service, token).await {
                    tracing::error!("Handler error for {}: {}", remote_addr, e);
                }
            });
        }

        // Signal all handlers to shut down gracefully
        cancel_token.cancel();
        tracing::info!("HttpSubsystem shutting down");
        Ok(())
    }
}

async fn handler(socket: TcpStream, remote_addr: SocketAddr, tower_service: Router<()>, cancel_token: CancellationToken) -> Result<(), Error> {
    let socket = TokioIo::new(socket);
    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| tower_service.clone().call(request));
    let conn = hyper::server::conn::http1::Builder::new().serve_connection(socket, hyper_service);
    let mut conn = std::pin::pin!(conn);

    tokio::select! {
        result = conn.as_mut() => {
            if let Err(err) = result {
                tracing::warn!("Failed to serve connection {}: {:#}", remote_addr, err);
            }
        }
        _ = cancel_token.cancelled() => {
            tracing::debug!("signal received, starting graceful shutdown");
        }
    }

    tracing::debug!("Connection {} closed", remote_addr);
    Ok(())
}

async fn hello_handler() -> Html<&'static str> {
    tracing::info!("Hello world!");
    Html("<h1>Hello, World!</h1>")
}
