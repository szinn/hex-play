use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use crate::{
    Error,
    models::{NewUser, User},
};

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
    async fn add_user(&self, transaction: &dyn Transaction, user: NewUser) -> Result<User, Error>;
    async fn update_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error>;
    async fn delete_user(&self, transaction: &dyn Transaction, id: i64) -> Result<User, Error>;
    async fn list_users(&self, transaction: &dyn Transaction, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error>;
    async fn find_by_id(&self, transaction: &dyn Transaction, id: i64) -> Result<Option<User>, Error>;
    async fn find_by_email(&self, transaction: &dyn Transaction, email: &str) -> Result<Option<User>, Error>;
}

pub struct RepositoryService {
    pub repository: Arc<dyn Repository>,
    pub user_service: Arc<dyn UserService>,
}

/// Execute a closure within a transaction, automatically committing on success
/// or rolling back on error.
///
/// # Example
/// ```ignore
/// let result = transaction(&*repository, |tx| Box::pin(async move {
///     // do stuff with tx
///     Ok(result)
/// })).await?;
/// ```
#[tracing::instrument(level = "trace", skip(repository, callback))]
pub async fn transaction<F, T>(repository: &dyn Repository, callback: F) -> Result<T, Error>
where
    F: for<'c> FnOnce(&'c dyn Transaction) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'c>> + Send,
    T: Send,
{
    let tx = repository.begin().await?;
    match callback(&*tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            // Best effort rollback - if it fails, we still return the original error
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Execute a closure within a read-only transaction.
///
/// # Example
/// ```ignore
/// let result = read_only_transaction(&*repository, |tx| Box::pin(async move {
///     // do stuff with tx
///     Ok(result)
/// })).await?;
/// ```
#[tracing::instrument(level = "trace", skip(repository, callback))]
pub async fn read_only_transaction<F, T>(repository: &dyn Repository, callback: F) -> Result<T, Error>
where
    F: for<'c> FnOnce(&'c dyn Transaction) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'c>> + Send,
    T: Send,
{
    let tx = repository.begin_read_only().await?;
    callback(&*tx).await
}
