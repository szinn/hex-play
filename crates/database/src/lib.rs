use std::sync::Arc;

use hex_play_core::{Error, repositories::RepositoryService};
use sea_orm::DatabaseConnection;

use crate::adapters::{user::UserRepositoryAdapter, user_info::UserInfoRepositoryAdapter};

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
    // database
    //     .get_schema_registry("hex-play-database::entities::*")
    //     .sync(&database)
    //     .await
    //     .map_err(handle_dberr)?;
    apply_migrations(&database).await?;

    let repository = RepositoryImpl::new(database);
    let user_repository = UserRepositoryAdapter::new();
    let user_info_repository = UserInfoRepositoryAdapter::new();

    let repository_service = RepositoryService {
        repository: Arc::new(repository),
        user_repository: Arc::new(user_repository),
        user_info_repository: Arc::new(user_info_repository),
    };

    Ok(Arc::new(repository_service))
}

#[cfg(test)]
pub mod test_support {
    use std::sync::Arc;

    use hex_play_core::repositories::RepositoryService;
    use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};

    use crate::{
        RepositoryImpl,
        adapters::{user::UserRepositoryAdapter, user_info::UserInfoRepositoryAdapter},
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
        let user_repository = UserRepositoryAdapter::new();
        let user_info_repository = UserInfoRepositoryAdapter::new();

        Arc::new(RepositoryService {
            repository: Arc::new(repository),
            user_repository: Arc::new(user_repository),
            user_info_repository: Arc::new(user_info_repository),
        })
    }
}
