use std::sync::Arc;

use hex_play_core::{Error, RepositoryService};
use sea_orm::{ConnectOptions, Database};

use crate::{adapters::user::UserServiceAdapter, handle_dberr};

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
        repository: Arc::new(repository),
        user_service: Arc::new(user_service),
    };

    Ok(Arc::new(repository_service))
}
