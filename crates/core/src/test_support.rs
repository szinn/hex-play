//! Test utilities for mocking core services.
//! Only compiled when the `test-support` feature is enabled.

use std::sync::{Arc, Mutex};

use crate::{
    CoreServices, Error,
    session::{
        SessionService,
        model::{NewSession, Session},
    },
    user::{
        UserService,
        model::{NewUser, User, UserId, UserToken},
    },
};

/// A mock implementation of [`UserService`] for testing.
///
/// Configure expected results using the builder methods, then use in tests.
///
/// # Example
/// ```ignore
/// let mock = MockUserService::default()
///     .with_find_by_id_result(Ok(Some(user)));
/// ```
#[derive(Default)]
pub struct MockUserService {
    pub add_user_result: Mutex<Option<Result<User, Error>>>,
    pub update_user_result: Mutex<Option<Result<User, Error>>>,
    pub delete_user_result: Mutex<Option<Result<User, Error>>>,
    pub find_by_id_result: Mutex<Option<Result<Option<User>, Error>>>,
    pub find_by_token_result: Mutex<Option<Result<Option<User>, Error>>>,
    pub list_users_result: Mutex<Option<Result<Vec<User>, Error>>>,
}

impl MockUserService {
    pub fn with_add_user_result(self, result: Result<User, Error>) -> Self {
        *self.add_user_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_update_user_result(self, result: Result<User, Error>) -> Self {
        *self.update_user_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_delete_user_result(self, result: Result<User, Error>) -> Self {
        *self.delete_user_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_find_by_id_result(self, result: Result<Option<User>, Error>) -> Self {
        *self.find_by_id_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_find_by_token_result(self, result: Result<Option<User>, Error>) -> Self {
        *self.find_by_token_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_list_users_result(self, result: Result<Vec<User>, Error>) -> Self {
        *self.list_users_result.lock().unwrap() = Some(result);
        self
    }
}

#[async_trait::async_trait]
impl UserService for MockUserService {
    async fn add_user(&self, _user: NewUser) -> Result<User, Error> {
        self.add_user_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("add_user")))
    }

    async fn update_user(&self, _user: User) -> Result<User, Error> {
        self.update_user_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("update_user")))
    }

    async fn list_users(&self, _start_id: Option<UserId>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
        self.list_users_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("list_users")))
    }

    async fn delete_user(&self, _id: UserId) -> Result<User, Error> {
        self.delete_user_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_user")))
    }

    async fn find_by_id(&self, _id: UserId) -> Result<Option<User>, Error> {
        self.find_by_id_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("find_by_id")))
    }

    async fn find_by_token(&self, _token: UserToken) -> Result<Option<User>, Error> {
        self.find_by_token_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("find_by_token")))
    }
}

/// A mock implementation of [`SessionService`] for testing.
#[derive(Default)]
pub struct MockSessionService {
    pub count_result: Mutex<Option<Result<i64, Error>>>,
    pub store_result: Mutex<Option<Result<Session, Error>>>,
    pub load_result: Mutex<Option<Result<Option<Session>, Error>>>,
    pub delete_by_id_result: Mutex<Option<Result<(), Error>>>,
    pub exists_result: Mutex<Option<Result<bool, Error>>>,
    pub delete_by_expiry_result: Mutex<Option<Result<Vec<String>, Error>>>,
    pub delete_all_result: Mutex<Option<Result<(), Error>>>,
    pub get_ids_result: Mutex<Option<Result<Vec<String>, Error>>>,
}

#[async_trait::async_trait]
impl SessionService for MockSessionService {
    async fn count(&self) -> Result<i64, Error> {
        self.count_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("count")))
    }
    async fn store(&self, _session: NewSession) -> Result<Session, Error> {
        self.store_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("store")))
    }
    async fn load(&self, _id: &str) -> Result<Option<Session>, Error> {
        self.load_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("load")))
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), Error> {
        self.delete_by_id_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_by_id")))
    }
    async fn exists(&self, _id: &str) -> Result<bool, Error> {
        self.exists_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("exists")))
    }
    async fn delete_by_expiry(&self) -> Result<Vec<String>, Error> {
        self.delete_by_expiry_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_by_expiry")))
    }
    async fn delete_all(&self) -> Result<(), Error> {
        self.delete_all_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_all")))
    }
    async fn get_ids(&self) -> Result<Vec<String>, Error> {
        self.get_ids_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("get_ids")))
    }
}

/// Creates a CoreServices instance with the given mock UserService.
///
/// This is a convenience function for tests that need a CoreServices.
pub fn create_core_services_with_mock(mock: MockUserService) -> CoreServices {
    CoreServices {
        user_service: Arc::new(mock),
        session_service: Arc::new(MockSessionService::default()),
    }
}

/// Creates an Arc-wrapped CoreServices instance with the given mock
/// UserService.
///
/// This is a convenience function for tests that need an Arc<CoreServices>.
pub fn create_arc_core_services_with_mock(mock: MockUserService) -> Arc<CoreServices> {
    Arc::new(create_core_services_with_mock(mock))
}
