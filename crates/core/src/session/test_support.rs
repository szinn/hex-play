use std::sync::Mutex;

use crate::{
    Error,
    session::{NewSession, Session, SessionService},
};

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
