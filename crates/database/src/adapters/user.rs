use chrono::Utc;
use hex_play_core::{
    Error, RepositoryError,
    models::{NewUser, User},
    services::{Transaction, UserService},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{
    entities::{prelude, users},
    error::handle_dberr,
    transaction::TransactionImpl,
};

impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            version: model.version,
            name: model.name,
            email: model.email,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}

pub struct UserServiceAdapter;

impl UserServiceAdapter {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceAdapter {
    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn add_user(&self, transaction: &dyn Transaction, user: NewUser) -> Result<User, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let model = users::ActiveModel {
            name: Set(user.name),
            email: Set(user.email),
            version: Set(0),
            ..Default::default()
        };

        let model = model.insert(transaction).await.map_err(handle_dberr)?;

        Ok(model.into())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn update_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error> {
        if user.id < 0 {
            return Err(Error::InvalidId(user.id));
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let existing = prelude::Users::find_by_id(user.id)
            .one(transaction)
            .await
            .map_err(handle_dberr)?
            .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;
        if existing.version != user.version {
            return Err(Error::RepositoryError(RepositoryError::Conflict));
        }

        let mut updater: users::ActiveModel = existing.clone().into();
        if existing.name != user.name {
            updater.name = Set(user.name);
        }
        if existing.email != user.email {
            updater.email = Set(user.email);
        }

        let updated = updater.update(transaction).await.map_err(handle_dberr)?;

        Ok(updated.into())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn delete_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let existing = prelude::Users::find_by_id(user.id).one(transaction).await.map_err(handle_dberr)?;
        let Some(existing) = existing else {
            return Err(Error::RepositoryError(RepositoryError::NotFound));
        };
        if existing.version != user.version {
            return Err(Error::RepositoryError(RepositoryError::Conflict));
        }

        let user: User = existing.clone().into();
        existing.delete(transaction).await.map_err(handle_dberr)?;

        Ok(user)
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn list_users(&self, transaction: &dyn Transaction, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        const DEFAULT_PAGE_SIZE: u64 = 50;
        const MAX_PAGE_SIZE: u64 = 50;

        if let Some(start_id) = start_id {
            if start_id < 0 {
                return Err(Error::InvalidId(start_id));
            }
        }

        if let Some(page_size) = page_size {
            if page_size < 1 {
                return Err(Error::InvalidPageSize(page_size));
            }
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let mut query = prelude::Users::find().order_by_asc(users::Column::Id);

        if let Some(start_id) = start_id {
            query = query.filter(users::Column::Id.gte(start_id));
        }

        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);
        query = query.limit(page_size);

        let users = query.all(transaction).await.map_err(handle_dberr)?;

        Ok(users.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn find_by_id(&self, transaction: &dyn Transaction, id: i64) -> Result<Option<User>, Error> {
        if id < 0 {
            return Err(Error::InvalidId(id));
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        Ok(prelude::Users::find_by_id(id).one(transaction).await.map_err(handle_dberr)?.map(Into::into))
    }
    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn find_by_email(&self, transaction: &dyn Transaction, email: &str) -> Result<Option<User>, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        Ok(prelude::Users::find_by_email(email)
            .one(transaction)
            .await
            .map_err(handle_dberr)?
            .map(Into::into))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use hex_play_core::{Error, RepositoryError, models::NewUser, services::UserService};
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    use super::UserServiceAdapter;
    use crate::{entities::users, test_support::create_mock_repository_service_with_db};

    fn create_test_user_model(id: i64, name: &str, email: &str) -> users::Model {
        users::Model {
            id,
            name: name.to_string(),
            email: email.to_string(),
            version: 0,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap().fixed_offset(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap().fixed_offset(),
        }
    }

    // ===================
    // Tests: add_user
    // ===================
    #[tokio::test]
    async fn test_add_user_success() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[create_test_user_model(1, "John Doe", "john@example.com")]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let new_user = NewUser {
            name: "John Doe".into(),
            email: "john@example.com".into(),
        };

        let result = repo_service.user_service.add_user(&*tx, new_user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
    }

    // ===================
    // Tests: find_by_id
    // ===================
    #[tokio::test]
    async fn test_find_by_id_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[create_test_user_model(1, "John Doe", "john@example.com")]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.find_by_id(&*tx, 1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([Vec::<users::Model>::new()]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.find_by_id(&*tx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_by_id_invalid_id() {
        let adapter = UserServiceAdapter::new();

        // Create a mock transaction - we won't actually use it since validation fails
        // first
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres);
        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = adapter.find_by_id(&*tx, -1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidId(-1)));
    }

    // ===================
    // Tests: find_by_email
    // ===================
    #[tokio::test]
    async fn test_find_by_email_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[create_test_user_model(1, "John Doe", "john@example.com")]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.find_by_email(&*tx, "john@example.com").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, "john@example.com");
    }

    #[tokio::test]
    async fn test_find_by_email_not_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([Vec::<users::Model>::new()]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.find_by_email(&*tx, "unknown@example.com").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ===================
    // Tests: list_users
    // ===================
    #[tokio::test]
    async fn test_list_users_success() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[
            create_test_user_model(1, "John Doe", "john@example.com"),
            create_test_user_model(2, "Jane Doe", "jane@example.com"),
        ]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.list_users(&*tx, None, None).await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "John Doe");
        assert_eq!(users[1].name, "Jane Doe");
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([Vec::<users::Model>::new()]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.list_users(&*tx, None, None).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_users_invalid_start_id() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres);
        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.list_users(&*tx, Some(-1), None).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidId(-1)));
    }

    #[tokio::test]
    async fn test_list_users_invalid_page_size() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres);
        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let result = repo_service.user_service.list_users(&*tx, None, Some(0)).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidPageSize(0)));
    }

    // ===================
    // Tests: update_user
    // ===================
    #[tokio::test]
    async fn test_update_user_success() {
        let existing = create_test_user_model(1, "John Doe", "john@example.com");
        let updated = create_test_user_model(1, "John Updated", "john.updated@example.com");

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[existing]])
            .append_query_results([[updated.clone()]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(1)
            .version(0)
            .name("John Updated".into())
            .email("john.updated@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.update_user(&*tx, user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "John Updated");
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([Vec::<users::Model>::new()]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(999)
            .version(0)
            .name("Nonexistent".into())
            .email("none@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.update_user(&*tx, user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn test_update_user_version_conflict() {
        let existing = create_test_user_model(1, "John Doe", "john@example.com");

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[existing]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(1)
            .version(99) // Wrong version
            .name("John Updated".into())
            .email("john@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.update_user(&*tx, user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::Conflict)));
    }

    #[tokio::test]
    async fn test_update_user_invalid_id() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres);
        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(-1)
            .version(0)
            .name("Invalid".into())
            .email("invalid@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.update_user(&*tx, user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidId(-1)));
    }

    // ===================
    // Tests: delete_user
    // ===================
    #[tokio::test]
    async fn test_delete_user_success() {
        let existing = create_test_user_model(1, "John Doe", "john@example.com");

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[existing.clone()]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(1)
            .version(0)
            .name("John Doe".into())
            .email("john@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.delete_user(&*tx, user).await;

        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert_eq!(deleted.id, 1);
        assert_eq!(deleted.name, "John Doe");
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([Vec::<users::Model>::new()]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(999)
            .version(0)
            .name("Nonexistent".into())
            .email("none@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.delete_user(&*tx, user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn test_delete_user_version_conflict() {
        let existing = create_test_user_model(1, "John Doe", "john@example.com");

        let mock_db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([[existing]]);

        let repo_service = create_mock_repository_service_with_db(mock_db);
        let tx = repo_service.repository.begin().await.unwrap();

        let user = hex_play_core::models::UserBuilder::default()
            .id(1)
            .version(99) // Wrong version
            .name("John Doe".into())
            .email("john@example.com".into())
            .build()
            .unwrap();

        let result = repo_service.user_service.delete_user(&*tx, user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::Conflict)));
    }
}
