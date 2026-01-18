use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;

use crate::error::Error;
use crate::error::handle_dberr;
use crate::migration::apply_migrations;

pub mod entities;
pub mod error;
pub mod migration;

pub async fn create_database_connection(database_url: &str) -> Result<DatabaseConnection, Error> {
    tracing::debug!("Connecting to database...");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let database = Database::connect(opt).await.map_err(handle_dberr)?;
    apply_migrations(&database).await?;

    Ok(database)
}
