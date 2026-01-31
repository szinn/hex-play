use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use crate::{
    Error,
    services::{UserInfoService, UserService},
};

/// Execute an async operation within a read-write transaction.
///
/// Clones the user_service, begins a transaction, executes the body,
/// and commits on success or rolls back on error.
///
/// # Example
/// ```ignore
/// with_transaction!(self, user_service, |tx| {
///     user_service.add_user(tx, user).await
/// })
/// ```
#[macro_export]
macro_rules! with_transaction {
    ($self:expr, $service:ident, |$tx:ident| $body:expr) => {{
        let $service = $self.repository_service.$service.clone();
        $crate::services::transaction(&*$self.repository_service.repository, |$tx| Box::pin(async move { $body })).await
    }};
}

/// Execute an async operation within a read-only transaction.
///
/// Clones the user_service and executes the body within a read-only
/// transaction.
///
/// # Example
/// ```ignore
/// with_read_only_transaction!(self, user_service, |tx| {
///     user_service.find_by_id(tx, id).await
/// })
/// ```
#[macro_export]
macro_rules! with_read_only_transaction {
    ($self:expr, $service:ident, |$tx:ident| $body:expr) => {{
        let $service = $self.repository_service.$service.clone();
        $crate::services::read_only_transaction(&*$self.repository_service.repository, |$tx| Box::pin(async move { $body })).await
    }};
}

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

pub struct RepositoryService {
    pub repository: Arc<dyn Repository>,
    pub user_service: Arc<dyn UserService>,
    pub user_info_service: Arc<dyn UserInfoService>,
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
