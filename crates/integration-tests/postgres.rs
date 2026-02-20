use std::sync::Arc;

use hex_play_core::CoreServices;
use hex_play_database::create_repository_service;
use sea_orm::Database;
use testcontainers::{ImageExt as _, runners::AsyncRunner as _};
use testcontainers_modules::postgres::Postgres;

pub async fn setup() -> (testcontainers::ContainerAsync<Postgres>, Arc<CoreServices>) {
    let container = Postgres::default().with_tag("17").start().await.unwrap();
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let db = Database::connect(&url).await.unwrap();
    let repository_service = create_repository_service(db).await.unwrap();
    let core_services = hex_play_core::create_services(repository_service).unwrap();

    (container, core_services)
}
