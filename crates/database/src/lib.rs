use std::sync::Arc;

use hex_play_core::Error;
use hex_play_core::RepositoryService;
use sea_orm::ConnectOptions;
use sea_orm::Database;

use crate::adapters::user::UserServiceAdapter;
use crate::handle_dberr;

pub mod error;
pub mod migration;

pub use error::*;
pub use migration::*;

mod adapters;
mod entities;
mod repository;
mod transaction;

use repository::*;
use transaction::*;

pub async fn create_repository_service(database_url: &str) -> Result<Arc<RepositoryService>, Error> {
    tracing::debug!("Connecting to database...");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let database = Database::connect(opt).await.map_err(handle_dberr)?;
    apply_migrations(&database).await?;

    let repository = RepositoryImpl::new(database);
    let user_service = UserServiceAdapter::new();

    let repository_service = RepositoryService {
        repository: Box::new(repository),
        user_service: Box::new(user_service),
    };

    Ok(Arc::new(repository_service))
}
