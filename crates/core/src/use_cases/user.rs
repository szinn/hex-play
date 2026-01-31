use std::sync::Arc;

use crate::{
    Error,
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
            Box::pin(async move { user_service.delete_user(tx, id).await })
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
