use std::sync::Arc;

use sea_orm::ConnectOptions;
use sea_orm::Database;

use crate::Error;
use crate::handle_dberr;

pub mod error;
pub mod migration;

pub use error::*;
pub use migration::*;

mod entities;
mod repository;
mod transaction;

use repository::*;
use transaction::*;

pub async fn create_repository(database_url: &str) -> Result<Arc<dyn Repository>, Error> {
    tracing::debug!("Connecting to database...");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let database = Database::connect(opt).await.map_err(handle_dberr)?;
    apply_migrations(&database).await?;

    Ok(Arc::new(RepositoryImpl::new(database)))
}
