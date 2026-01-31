use std::sync::Arc;

use hex_play_core::{Error, services::RepositoryService};
use sea_orm::DatabaseConnection;

use crate::adapters::{user::UserServiceAdapter, user_info::UserInfoServiceAdapter};

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
    let user_info_service = UserInfoServiceAdapter::new();

    let repository_service = RepositoryService {
        repository: Arc::new(repository),
        user_service: Arc::new(user_service),
        user_info_service: Arc::new(user_info_service),
    };

    Ok(Arc::new(repository_service))
}

#[cfg(test)]
pub mod test_support {
    use std::sync::Arc;

    use hex_play_core::services::RepositoryService;
    use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};

    use crate::{
        RepositoryImpl,
        adapters::{user::UserServiceAdapter, user_info::UserInfoServiceAdapter},
    };

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
        let repository = RepositoryImpl::new(database);
        let user_service = UserServiceAdapter::new();
        let user_info_service = UserInfoServiceAdapter::new();

        Arc::new(RepositoryService {
            repository: Arc::new(repository),
            user_service: Arc::new(user_service),
            user_info_service: Arc::new(user_info_service),
        })
    }
}
