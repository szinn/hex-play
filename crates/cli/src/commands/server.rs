use anyhow::Context;
use hex_play_core::UserBuilder;
use hex_play_database::create_repository_service;

use crate::config::Config;

pub async fn run_server_command(config: &Config) -> anyhow::Result<()> {
    let crate_version = clap::crate_version!();

    tracing::info!("HexPlay {}", crate_version);

    let _server = {
        let _span_ = tracing::span!(tracing::Level::TRACE, "CreateServer").entered();
        let repository_service = create_repository_service(&config.database.database_url)
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

        let tx = repository_service.repository.begin().await?;
        let user = UserBuilder::default()
            .name("Fred Wombat".into())
            .email("fred@wombat.com".into())
            .build()
            .unwrap();
        dbg!(&user);

        let existing_user = repository_service.user_service.find_by_email(&*tx, &user.email).await?;
        let user = if let Some(user) = existing_user {
            tracing::info!("Found user");
            let user = UserBuilder::default()
                .id(user.id)
                .version(user.version)
                .name(user.name)
                .email("also_fred@wombat.com".into())
                .created_at(user.created_at)
                .updated_at(user.updated_at)
                .build()
                .unwrap();
            user
        } else {
            tracing::info!("Not found");
            repository_service.user_service.add_user(&*tx, user).await?
        };

        // let user = repository_service.user_service.add_user(&*tx, user).await?;
        dbg!(&user);

        tx.commit().await?;

        _ = repository_service.repository.close().await;

        5
    };

    // server.await?;

    Ok(())
}
