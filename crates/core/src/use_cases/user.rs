use std::sync::Arc;

use crate::{
    Error, RepositoryError,
    models::{NewUser, User},
    services::{RepositoryService, read_only_transaction, transaction},
};

#[async_trait::async_trait]
pub trait UserUseCases: Send + Sync {
    async fn add_user(&self, user: NewUser) -> Result<User, Error>;
    async fn update_user(&self, user: User) -> Result<User, Error>;
    async fn list_users(&self, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error>;
    async fn delete_user(&self, id: i64) -> Result<User, Error>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
}

pub(crate) struct UserUseCasesImpl {
    repository_service: Arc<RepositoryService>,
}

impl UserUseCasesImpl {
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self { repository_service }
    }
}

#[async_trait::async_trait]
impl UserUseCases for UserUseCasesImpl {
    #[tracing::instrument(level = "trace", skip(self, user))]
    async fn add_user(&self, user: NewUser) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move { user_service.add_user(tx, user).await })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self, user))]
    async fn update_user(&self, user: User) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move { user_service.update_user(tx, user).await })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn list_users(&self, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        let user_service = self.repository_service.user_service.clone();
        read_only_transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move { user_service.list_users(tx, start_id, page_size).await })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn delete_user(&self, id: i64) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let user = user_service
                    .find_by_id(tx, id)
                    .await?
                    .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;
                user_service.delete_user(tx, user).await
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error> {
        let user_service = self.repository_service.user_service.clone();
        read_only_transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move { user_service.find_by_id(tx, id).await })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };

    use super::{UserUseCases, UserUseCasesImpl};
    use crate::{
        Error, RepositoryError,
        models::{NewUser, User, UserBuilder},
        services::{Repository, RepositoryService, Transaction, UserService},
    };

    // ===================
    // Mock Transaction
    // ===================
    struct MockTransaction;

    #[async_trait::async_trait]
    impl Transaction for MockTransaction {
        fn as_any(&self) -> &dyn Any {
            self
        }

        async fn commit(self: Box<Self>) -> Result<(), Error> {
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> Result<(), Error> {
            Ok(())
        }
    }

    // ===================
    // Mock Repository
    // ===================
    struct MockRepository;

    #[async_trait::async_trait]
    impl Repository for MockRepository {
        async fn begin(&self) -> Result<Box<dyn Transaction>, Error> {
            Ok(Box::new(MockTransaction))
        }

        async fn begin_read_only(&self) -> Result<Box<dyn Transaction>, Error> {
            Ok(Box::new(MockTransaction))
        }

        async fn close(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    // ===================
    // Mock UserService
    // ===================
    #[derive(Default)]
    struct MockUserService {
        add_user_result: Mutex<Option<Result<User, Error>>>,
        update_user_result: Mutex<Option<Result<User, Error>>>,
        delete_user_result: Mutex<Option<Result<User, Error>>>,
        find_by_id_result: Mutex<Option<Result<Option<User>, Error>>>,
        list_users_result: Mutex<Option<Result<Vec<User>, Error>>>,
    }

    impl MockUserService {
        fn with_add_user_result(self, result: Result<User, Error>) -> Self {
            *self.add_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_update_user_result(self, result: Result<User, Error>) -> Self {
            *self.update_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_user_result(self, result: Result<User, Error>) -> Self {
            *self.delete_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_id_result(self, result: Result<Option<User>, Error>) -> Self {
            *self.find_by_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_list_users_result(self, result: Result<Vec<User>, Error>) -> Self {
            *self.list_users_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait::async_trait]
    impl UserService for MockUserService {
        async fn add_user(&self, _tx: &dyn Transaction, _user: NewUser) -> Result<User, Error> {
            self.add_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn update_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            self.update_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn delete_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            self.delete_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn list_users(&self, _tx: &dyn Transaction, _start_id: Option<i64>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
            self.list_users_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn find_by_id(&self, _tx: &dyn Transaction, _id: i64) -> Result<Option<User>, Error> {
            self.find_by_id_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn find_by_email(&self, _tx: &dyn Transaction, _email: &str) -> Result<Option<User>, Error> {
            Err(Error::Message("Not implemented in mock".into()))
        }
    }

    // ===================
    // Test Helpers
    // ===================
    fn create_test_user(id: i64, name: &str, email: &str) -> User {
        UserBuilder::default().id(id).version(0).name(name.into()).email(email.into()).build().unwrap()
    }

    fn create_use_cases(mock_user_service: MockUserService) -> UserUseCasesImpl {
        let repository_service = Arc::new(RepositoryService {
            repository: Arc::new(MockRepository),
            user_service: Arc::new(mock_user_service),
        });
        UserUseCasesImpl::new(repository_service)
    }

    // ===================
    // Tests: add_user
    // ===================
    #[tokio::test]
    async fn test_add_user_success() {
        let expected_user = create_test_user(1, "John Doe", "john@example.com");
        let mock_service = MockUserService::default().with_add_user_result(Ok(expected_user.clone()));
        let use_cases = create_use_cases(mock_service);

        let new_user = NewUser {
            name: "John Doe".into(),
            email: "john@example.com".into(),
        };

        let result = use_cases.add_user(new_user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
    }

    #[tokio::test]
    async fn test_add_user_propagates_error() {
        let mock_service = MockUserService::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate email".into()))));
        let use_cases = create_use_cases(mock_service);

        let new_user = NewUser {
            name: "John Doe".into(),
            email: "john@example.com".into(),
        };

        let result = use_cases.add_user(new_user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::Constraint(_))));
    }

    // ===================
    // Tests: update_user
    // ===================
    #[tokio::test]
    async fn test_update_user_success() {
        let updated_user = create_test_user(1, "John Updated", "john.updated@example.com");
        let mock_service = MockUserService::default().with_update_user_result(Ok(updated_user.clone()));
        let use_cases = create_use_cases(mock_service);

        let user = create_test_user(1, "John Doe", "john@example.com");

        let result = use_cases.update_user(user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "John Updated");
        assert_eq!(user.email, "john.updated@example.com");
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mock_service = MockUserService::default().with_update_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let use_cases = create_use_cases(mock_service);

        let user = create_test_user(999, "Nonexistent", "none@example.com");

        let result = use_cases.update_user(user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }

    // ===================
    // Tests: find_by_id
    // ===================
    #[tokio::test]
    async fn test_find_by_id_found() {
        let expected_user = create_test_user(1, "John Doe", "john@example.com");
        let mock_service = MockUserService::default().with_find_by_id_result(Ok(Some(expected_user.clone())));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.find_by_id(1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mock_service = MockUserService::default().with_find_by_id_result(Ok(None));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.find_by_id(999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ===================
    // Tests: list_users
    // ===================
    #[tokio::test]
    async fn test_list_users_success() {
        let users = vec![
            create_test_user(1, "John Doe", "john@example.com"),
            create_test_user(2, "Jane Doe", "jane@example.com"),
        ];
        let mock_service = MockUserService::default().with_list_users_result(Ok(users));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.list_users(None, None).await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "John Doe");
        assert_eq!(users[1].name, "Jane Doe");
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let mock_service = MockUserService::default().with_list_users_result(Ok(vec![]));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.list_users(None, None).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ===================
    // Tests: delete_user
    // ===================
    #[tokio::test]
    async fn test_delete_user_success() {
        let user_to_delete = create_test_user(1, "John Doe", "john@example.com");
        let mock_service = MockUserService::default()
            .with_find_by_id_result(Ok(Some(user_to_delete.clone())))
            .with_delete_user_result(Ok(user_to_delete.clone()));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.delete_user(1).await;

        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert_eq!(deleted.id, 1);
        assert_eq!(deleted.name, "John Doe");
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mock_service = MockUserService::default().with_find_by_id_result(Ok(None));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.delete_user(999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }
}
