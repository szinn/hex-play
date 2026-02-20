use std::sync::Arc;

use crate::{
    Error,
    repository::RepositoryService,
    session::{NewSession, Session},
    with_transaction,
};

#[async_trait::async_trait]
pub trait SessionService: Send + Sync {
    async fn count(&self) -> Result<i64, Error>;
    async fn store(&self, session: NewSession) -> Result<Session, Error>;
    async fn load(&self, id: &str) -> Result<Option<Session>, Error>;
    async fn delete_by_id(&self, id: &str) -> Result<(), Error>;
    async fn exists(&self, id: &str) -> Result<bool, Error>;
    async fn delete_by_expiry(&self) -> Result<Vec<String>, Error>;
    async fn delete_all(&self) -> Result<(), Error>;
    async fn get_ids(&self) -> Result<Vec<String>, Error>;
}

pub(crate) struct SessionServiceImpl {
    repository_service: Arc<RepositoryService>,
}

impl SessionServiceImpl {
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self { repository_service }
    }
}

#[async_trait::async_trait]
impl SessionService for SessionServiceImpl {
    async fn count(&self) -> Result<i64, Error> {
        with_transaction!(self, session_repository, |tx| session_repository.count(tx).await)
    }

    async fn store(&self, session: NewSession) -> Result<Session, Error> {
        with_transaction!(self, session_repository, |tx| session_repository.store(tx, session).await)
    }

    async fn load(&self, id: &str) -> Result<Option<Session>, Error> {
        let id = id.to_owned();
        with_transaction!(self, session_repository, |tx| session_repository.load(tx, &id).await)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), Error> {
        let id = id.to_owned();
        with_transaction!(self, session_repository, |tx| session_repository.delete_by_id(tx, &id).await)
    }
    async fn exists(&self, id: &str) -> Result<bool, Error> {
        let id = id.to_owned();
        with_transaction!(self, session_repository, |tx| session_repository.exists(tx, &id).await)
    }
    async fn delete_by_expiry(&self) -> Result<Vec<String>, Error> {
        with_transaction!(self, session_repository, |tx| session_repository.delete_by_expiry(tx).await)
    }
    async fn delete_all(&self) -> Result<(), Error> {
        with_transaction!(self, session_repository, |tx| session_repository.delete_all(tx).await)
    }
    async fn get_ids(&self) -> Result<Vec<String>, Error> {
        with_transaction!(self, session_repository, |tx| session_repository.get_ids(tx).await)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };

    use chrono::{Duration, Utc};

    use super::{SessionService, SessionServiceImpl};
    use crate::{
        Error,
        repository::{Repository, RepositoryServiceBuilder, Transaction},
        session::{
            model::{NewSession, Session, SessionBuilder},
            repository::SessionRepository,
        },
        types::Email,
        user::{
            model::{NewUser, User, UserId, UserToken},
            repository::UserRepository,
        },
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
    // Mock UserRepository
    // ===================
    struct MockUserRepository;

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn add_user(&self, _tx: &dyn Transaction, _user: NewUser) -> Result<User, Error> {
            unimplemented!()
        }
        async fn update_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            unimplemented!()
        }
        async fn delete_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            unimplemented!()
        }
        async fn list_users(&self, _tx: &dyn Transaction, _start_id: Option<UserId>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _tx: &dyn Transaction, _id: UserId) -> Result<Option<User>, Error> {
            unimplemented!()
        }
        async fn find_by_email(&self, _tx: &dyn Transaction, _email: &Email) -> Result<Option<User>, Error> {
            unimplemented!()
        }
        async fn find_by_token(&self, _tx: &dyn Transaction, _token: UserToken) -> Result<Option<User>, Error> {
            unimplemented!()
        }
    }

    // ===================
    // Mock SessionRepository
    // ===================
    #[derive(Default)]
    struct MockSessionRepository {
        count_result: Mutex<Option<Result<i64, Error>>>,
        store_result: Mutex<Option<Result<Session, Error>>>,
        load_result: Mutex<Option<Result<Option<Session>, Error>>>,
        delete_by_id_result: Mutex<Option<Result<(), Error>>>,
        exists_result: Mutex<Option<Result<bool, Error>>>,
        delete_by_expiry_result: Mutex<Option<Result<Vec<String>, Error>>>,
        delete_all_result: Mutex<Option<Result<(), Error>>>,
        get_ids_result: Mutex<Option<Result<Vec<String>, Error>>>,
    }

    impl MockSessionRepository {
        fn with_count_result(self, result: Result<i64, Error>) -> Self {
            *self.count_result.lock().unwrap() = Some(result);
            self
        }

        fn with_store_result(self, result: Result<Session, Error>) -> Self {
            *self.store_result.lock().unwrap() = Some(result);
            self
        }

        fn with_load_result(self, result: Result<Option<Session>, Error>) -> Self {
            *self.load_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_by_id_result(self, result: Result<(), Error>) -> Self {
            *self.delete_by_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_exists_result(self, result: Result<bool, Error>) -> Self {
            *self.exists_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_by_expiry_result(self, result: Result<Vec<String>, Error>) -> Self {
            *self.delete_by_expiry_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_all_result(self, result: Result<(), Error>) -> Self {
            *self.delete_all_result.lock().unwrap() = Some(result);
            self
        }

        fn with_get_ids_result(self, result: Result<Vec<String>, Error>) -> Self {
            *self.get_ids_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn count(&self, _tx: &dyn Transaction) -> Result<i64, Error> {
            self.count_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("count")))
        }
        async fn store(&self, _tx: &dyn Transaction, _session: NewSession) -> Result<Session, Error> {
            self.store_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("store")))
        }
        async fn load(&self, _tx: &dyn Transaction, _id: &str) -> Result<Option<Session>, Error> {
            self.load_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("load")))
        }
        async fn delete_by_id(&self, _tx: &dyn Transaction, _id: &str) -> Result<(), Error> {
            self.delete_by_id_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_by_id")))
        }
        async fn exists(&self, _tx: &dyn Transaction, _id: &str) -> Result<bool, Error> {
            self.exists_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("exists")))
        }
        async fn delete_by_expiry(&self, _tx: &dyn Transaction) -> Result<Vec<String>, Error> {
            self.delete_by_expiry_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_by_expiry")))
        }
        async fn delete_all(&self, _tx: &dyn Transaction) -> Result<(), Error> {
            self.delete_all_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_all")))
        }
        async fn get_ids(&self, _tx: &dyn Transaction) -> Result<Vec<String>, Error> {
            self.get_ids_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(Error::MockNotConfigured("get_ids")))
        }
    }

    // ===================
    // Test Helpers
    // ===================
    fn create_use_cases(mock_session_repository: MockSessionRepository) -> SessionServiceImpl {
        let repository_service = Arc::new(
            RepositoryServiceBuilder::default()
                .repository(Arc::new(MockRepository) as Arc<dyn Repository>)
                .user_repository(Arc::new(MockUserRepository) as Arc<dyn UserRepository>)
                .session_repository(Arc::new(mock_session_repository) as Arc<dyn SessionRepository>)
                .build()
                .expect("All required fields provided"),
        );
        SessionServiceImpl::new(repository_service)
    }

    fn fake_session(id: &str) -> Session {
        SessionBuilder::default()
            .id(id.to_string())
            .session("session-data".to_string())
            .expires_at(Utc::now() + Duration::hours(1))
            .build()
            .unwrap()
    }

    // ===================
    // Tests: count
    // ===================
    #[tokio::test]
    async fn test_count_success() {
        let mock = MockSessionRepository::default().with_count_result(Ok(5));
        let svc = create_use_cases(mock);

        let result = svc.count().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[tokio::test]
    async fn test_count_zero() {
        let mock = MockSessionRepository::default().with_count_result(Ok(0));
        let svc = create_use_cases(mock);

        let result = svc.count().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ===================
    // Tests: store
    // ===================
    #[tokio::test]
    async fn test_store_success() {
        let expected = fake_session("sess-1");
        let mock = MockSessionRepository::default().with_store_result(Ok(expected.clone()));
        let svc = create_use_cases(mock);

        let new_session = NewSession::new("sess-1", "session-data", Utc::now() + Duration::hours(1)).unwrap();
        let result = svc.store(new_session).await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.id, "sess-1");
        assert_eq!(session.session, "session-data");
    }

    #[tokio::test]
    async fn test_store_propagates_error() {
        let mock = MockSessionRepository::default().with_store_result(Err(Error::Infrastructure("db error".into())));
        let svc = create_use_cases(mock);

        let new_session = NewSession::new("sess-1", "session-data", Utc::now() + Duration::hours(1)).unwrap();
        let result = svc.store(new_session).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: load
    // ===================
    #[tokio::test]
    async fn test_load_found() {
        let expected = fake_session("sess-1");
        let mock = MockSessionRepository::default().with_load_result(Ok(Some(expected.clone())));
        let svc = create_use_cases(mock);

        let result = svc.load("sess-1").await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().id, "sess-1");
    }

    #[tokio::test]
    async fn test_load_not_found() {
        let mock = MockSessionRepository::default().with_load_result(Ok(None));
        let svc = create_use_cases(mock);

        let result = svc.load("nonexistent").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ===================
    // Tests: delete_by_id
    // ===================
    #[tokio::test]
    async fn test_delete_by_id_success() {
        let mock = MockSessionRepository::default().with_delete_by_id_result(Ok(()));
        let svc = create_use_cases(mock);

        let result = svc.delete_by_id("sess-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_by_id_propagates_error() {
        let mock = MockSessionRepository::default().with_delete_by_id_result(Err(Error::Infrastructure("db error".into())));
        let svc = create_use_cases(mock);

        let result = svc.delete_by_id("sess-1").await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: exists
    // ===================
    #[tokio::test]
    async fn test_exists_true() {
        let mock = MockSessionRepository::default().with_exists_result(Ok(true));
        let svc = create_use_cases(mock);

        let result = svc.exists("sess-1").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_exists_false() {
        let mock = MockSessionRepository::default().with_exists_result(Ok(false));
        let svc = create_use_cases(mock);

        let result = svc.exists("nonexistent").await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // ===================
    // Tests: delete_by_expiry
    // ===================
    #[tokio::test]
    async fn test_delete_by_expiry_success() {
        let expired_ids = vec!["sess-1".to_string(), "sess-2".to_string()];
        let mock = MockSessionRepository::default().with_delete_by_expiry_result(Ok(expired_ids.clone()));
        let svc = create_use_cases(mock);

        let result = svc.delete_by_expiry().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expired_ids);
    }

    #[tokio::test]
    async fn test_delete_by_expiry_none_expired() {
        let mock = MockSessionRepository::default().with_delete_by_expiry_result(Ok(vec![]));
        let svc = create_use_cases(mock);

        let result = svc.delete_by_expiry().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ===================
    // Tests: delete_all
    // ===================
    #[tokio::test]
    async fn test_delete_all_success() {
        let mock = MockSessionRepository::default().with_delete_all_result(Ok(()));
        let svc = create_use_cases(mock);

        let result = svc.delete_all().await;

        assert!(result.is_ok());
    }

    // ===================
    // Tests: get_ids
    // ===================
    #[tokio::test]
    async fn test_get_ids_success() {
        let ids = vec!["sess-1".to_string(), "sess-2".to_string(), "sess-3".to_string()];
        let mock = MockSessionRepository::default().with_get_ids_result(Ok(ids.clone()));
        let svc = create_use_cases(mock);

        let result = svc.get_ids().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ids);
    }

    #[tokio::test]
    async fn test_get_ids_empty() {
        let mock = MockSessionRepository::default().with_get_ids_result(Ok(vec![]));
        let svc = create_use_cases(mock);

        let result = svc.get_ids().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
