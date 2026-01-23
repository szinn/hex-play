use anyhow::Context;
use hex_play_database::create_repository;

use crate::config::Config;

pub async fn run_server_command(config: &Config) -> anyhow::Result<()> {
    let crate_version = clap::crate_version!();

    tracing::info!("HexPlay {}", crate_version);

    let _server = {
        let _span_ = tracing::span!(tracing::Level::TRACE, "CreateServer").entered();
        let repository = create_repository(&config.database.database_url)
            .await
            .context("Couldn't create database connection")?;
        // let services = create_services(config, database).await.context("Couldn't
        // start services")?; let server = create_server(config,
        // services).await.context("Couldn't create TeslaSpy server")?;
        // let backend = create_backend(config,
        // server.gateways.clone()).context("Couldn't create backend")?;
        //
        // Toplevel::new(|s| async move {
        //     s.start(SubsystemBuilder::new("TeslaSpyServer", |h|
        // tesla_spy_core::run(server, h)));     s.start(SubsystemBuilder::new("
        // Backend", |h| tesla_spy_backend::run(backend, h))); })
        // .catch_signals()
        // .handle_shutdown_requests(Duration::from_secs(5))

        let tx = repository.begin().await?;
        tx.commit().await?;

        _ = repository.close().await;

        5
    };

    // server.await?;

    Ok(())
}
