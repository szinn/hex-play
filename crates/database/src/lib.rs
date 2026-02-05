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

#[cfg(test)]
pub mod test_support {
    use std::sync::Arc;

    use hex_play_core::repositories::{Repository, RepositoryService, RepositoryServiceBuilder, UserRepository};
    use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};

    use crate::{RepositoryImpl, adapters::user::UserRepositoryAdapter};

    /// Creates a RepositoryService with a mock database for testing.
    pub fn create_mock_repository_service() -> Arc<RepositoryService> {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres);
        let database = mock_db.into_connection();
        create_repository_service_from_connection(database)
    }

    /// Creates a RepositoryService from a pre-configured mock database.
    pub fn create_mock_repository_service_with_db(mock_db: MockDatabase) -> Arc<RepositoryService> {
        let database = mock_db.into_connection();
        create_repository_service_from_connection(database)
    }

    fn create_repository_service_from_connection(database: DatabaseConnection) -> Arc<RepositoryService> {
        Arc::new(
            RepositoryServiceBuilder::default()
                .repository(Arc::new(RepositoryImpl::new(database)) as Arc<dyn Repository>)
                .user_repository(Arc::new(UserRepositoryAdapter::new()) as Arc<dyn UserRepository>)
                .build()
                .expect("All required fields provided"),
        )
    }
}
