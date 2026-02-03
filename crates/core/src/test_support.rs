//! Test utilities for mocking core services.
//! Only compiled when the `test-support` feature is enabled.

use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::{
    Error,
    models::{NewUser, User},
    services::{CoreServices, UserService},
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

    async fn list_users(&self, _start_id: Option<i64>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
        self.list_users_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("list_users")))
    }

    async fn delete_user(&self, _id: i64) -> Result<User, Error> {
        self.delete_user_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("delete_user")))
    }

    async fn find_by_id(&self, _id: i64) -> Result<Option<User>, Error> {
        self.find_by_id_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("find_by_id")))
    }

    async fn find_by_token(&self, _token: Uuid) -> Result<Option<User>, Error> {
        self.find_by_token_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Err(Error::MockNotConfigured("find_by_token")))
    }
}

/// Creates a CoreServices instance with the given mock UserService.
///
/// This is a convenience function for tests that need a CoreServices.
pub fn create_core_services_with_mock(mock: MockUserService) -> CoreServices {
    CoreServices { user_service: Arc::new(mock) }
}

/// Creates an Arc-wrapped CoreServices instance with the given mock
/// UserService.
///
/// This is a convenience function for tests that need an Arc<CoreServices>.
pub fn create_arc_core_services_with_mock(mock: MockUserService) -> Arc<CoreServices> {
    Arc::new(create_core_services_with_mock(mock))
}
