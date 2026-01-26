use anyhow::Context;
use hex_play_api::create_api_subsystem;
use hex_play_core::create_services;
use hex_play_database::create_repository_service;
use sea_orm::{ConnectOptions, Database};
use tokio::time::Duration;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemBuilder, SubsystemHandle, Toplevel};

use crate::config::Config;

pub async fn run_server_command(config: &Config) -> anyhow::Result<()> {
    let crate_version = clap::crate_version!();

    tracing::info!("HexPlay {}", crate_version);

    let span = tracing::span!(tracing::Level::TRACE, "CreateServer").entered();

    let mut opt = ConnectOptions::new(&config.database.database_url);
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let database = Database::connect(opt).await.context("Couldn't create database connection")?;
    let repository_service = create_repository_service(database).await.context("Couldn't create database connection")?;

    let server = {
        let services = create_services(repository_service.clone()).context("Couldn't create core services")?;
        let api_subsystem = create_api_subsystem(services.clone());

        Toplevel::new(async |s: &mut SubsystemHandle| {
            s.start(SubsystemBuilder::new("Api", api_subsystem.into_subsystem()));
        })
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
    };

    span.exit();

    server.await?;

    // let tx = repository_service.repository.begin().await?;
    // let user = UserBuilder::default()
    //     .name("Fred Wombat".into())
    //     .email("fred@wombat.com".into())
    //     .build()
    //     .unwrap();
    // dbg!(&user);
    //
    // let existing_user = repository_service.user_service.find_by_email(&*tx,
    // &user.email).await?; let user = if let Some(user) = existing_user {
    //     tracing::info!("Found user");
    //     let mut user = user;
    //     user.email = "also_fred@wombat.com".into();
    //     let user = repository_service.user_service.update_user(&*tx,
    // user).await?;     dbg!(&user);
    //     user
    // } else {
    //     tracing::info!("Not found");
    //     repository_service.user_service.add_user(&*tx, user).await?
    // };
    // dbg!(&user);
    //
    // tx.commit().await?;
    //
    // let user_service = repository_service.user_service.clone();
    // let mary = transaction(&*repository_service.repository, |tx| {
    //     Box::pin(async move {
    //         let user = UserBuilder::default()
    //             .name("Mary Wombat".into())
    //             .email("mary@wombat.com".into())
    //             .build()
    //             .unwrap();
    //
    //         let existing_user = user_service.find_by_email(tx,
    // &user.email).await?;         let mary = if let Some(mary) = existing_user
    // {             tracing::info!("Mary already exists in the database");
    //             mary
    //         } else {
    //             user_service.add_user(tx, user).await?
    //         };
    //         Ok(mary)
    //     })
    // })
    // .await?;
    // dbg!(&mary);
    //
    // let user_service = repository_service.user_service.clone();
    // let bill = read_only_transaction(&*repository_service.repository, |tx| {
    //     Box::pin(async move {
    //         let user = UserBuilder::default()
    //             .name("Bill Wombat".into())
    //             .email("bill@wombat.com".into())
    //             .build()
    //             .unwrap();
    //
    //         let existing_user = user_service.find_by_email(tx,
    // &user.email).await?;         let bill = if let Some(bill) = existing_user
    // {             tracing::info!("Bill already exists in the database");
    //             bill
    //         } else {
    //             user
    //         };
    //         Ok(bill)
    //     })
    // })
    // .await?;
    // dbg!(&bill);

    repository_service.repository.close().await.context("Couldn't close database")?;

    Ok(())
}
