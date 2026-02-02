//! Test utilities for mocking core services.
//! Only compiled when the `test-support` feature is enabled.

use std::sync::Mutex;

use uuid::Uuid;

use crate::{
    Error,
    models::{NewUser, User},
    services::UserService,
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
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }

    async fn update_user(&self, _user: User) -> Result<User, Error> {
        self.update_user_result
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }

    async fn list_users(&self, _start_id: Option<i64>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
        self.list_users_result
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }

    async fn delete_user(&self, _id: i64) -> Result<User, Error> {
        self.delete_user_result
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }

    async fn find_by_id(&self, _id: i64) -> Result<Option<User>, Error> {
        self.find_by_id_result
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }

    async fn find_by_token(&self, _token: Uuid) -> Result<Option<User>, Error> {
        self.find_by_token_result
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
    }
}
