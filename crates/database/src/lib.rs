use std::sync::Arc;

use hex_play_core::{
    Error,
    repositories::{Repository, RepositoryService, RepositoryServiceBuilder, UserRepository},
};
use sea_orm::DatabaseConnection;

use crate::adapters::user::UserRepositoryAdapter;

pub mod error;

pub use error::*;

mod adapters;
mod entities;
mod repository;
mod transaction;

use repository::*;
use transaction::*;

#[tracing::instrument(level = "trace", skip(database))]
pub async fn create_repository_service(database: DatabaseConnection) -> Result<Arc<RepositoryService>, Error> {
    tracing::debug!("Connecting to database...");
    database
        .get_schema_registry("hex-play-database::entities::*")
        .sync(&database)
        .await
        .map_err(handle_dberr)?;

    let repository_service = RepositoryServiceBuilder::default()
        .repository(Arc::new(RepositoryImpl::new(database)) as Arc<dyn Repository>)
        .user_repository(Arc::new(UserRepositoryAdapter::new()) as Arc<dyn UserRepository>)
        .build()
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    Ok(Arc::new(repository_service))
}
