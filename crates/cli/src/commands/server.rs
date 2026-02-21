use anyhow::Context;
use hex_play_api::create_api_subsystem;
use hex_play_core::create_services;
use hex_play_database::{create_repository_service, open_database};
use hex_play_frontend::server::launch_server_frontend;
use tokio::time::Duration;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemBuilder, SubsystemHandle, Toplevel};

use crate::config::Config;

pub async fn run_server_command(config: &Config) -> anyhow::Result<()> {
    let crate_version = clap::crate_version!();

    tracing::info!("HexPlay {}", crate_version);

    let span = tracing::span!(tracing::Level::TRACE, "CreateServer").entered();

    let database = open_database(&config.database).await.context("Couldn't create database connection")?;
    let repository_service = create_repository_service(database).await.context("Couldn't create database connection")?;

    let server = {
        let services = create_services(repository_service.clone()).context("Couldn't create core services")?;
        let api_subsystem = create_api_subsystem(services.clone());

        launch_server_frontend(services.clone());

        Toplevel::new(async |s: &mut SubsystemHandle| {
            s.start(SubsystemBuilder::new("Api", api_subsystem.into_subsystem()));
        })
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
    };

    span.exit();

    server.await?;

    repository_service.repository().close().await.context("Couldn't close database")?;

    Ok(())
}
