use std::sync::Arc;

use hex_play_core::{Error, services::RepositoryService};
use sea_orm::DatabaseConnection;

use crate::adapters::user::UserServiceAdapter;

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

#[tracing::instrument(level = "trace", skip(database))]
pub async fn create_repository_service(database: DatabaseConnection) -> Result<Arc<RepositoryService>, Error> {
    tracing::debug!("Connecting to database...");
    apply_migrations(&database).await?;

    let repository = RepositoryImpl::new(database);
    let user_service = UserServiceAdapter::new();

    let repository_service = RepositoryService {
        repository: Arc::new(repository),
        user_service: Arc::new(user_service),
    };

    Ok(Arc::new(repository_service))
}
