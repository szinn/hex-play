use std::any::Any;

use crate::{Error, User};

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn Transaction>, Error>;
    async fn begin_read_only(&self) -> Result<Box<dyn Transaction>, Error>;
    async fn close(&self) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait Transaction: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    async fn commit(self: Box<Self>) -> Result<(), Error>;
    async fn rollback(self: Box<Self>) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    async fn add_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error>;
    async fn find_by_email(&self, transaction: &dyn Transaction, email: &str) -> Result<Option<User>, Error>;
}

pub struct RepositoryService {
    pub repository: Box<dyn Repository>,
    pub user_service: Box<dyn UserService>,
}
