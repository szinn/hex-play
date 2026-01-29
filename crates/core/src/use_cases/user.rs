use std::sync::Arc;

use crate::{
    Error,
    models::{NewUser, User},
    services::RepositoryService,
};

#[async_trait::async_trait]
pub trait UserUseCases: Send + Sync {
    async fn add_user(&self, user: NewUser) -> Result<User, Error>;
    async fn update_user(&self, user: User) -> Result<User, Error>;
    async fn list_users(&self, start_id: Option<usize>, page_size: Option<usize>) -> Result<Vec<User>, Error>;
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
    async fn add_user(&self, _user: NewUser) -> Result<User, Error> {
        Err(Error::Message("Not implemented".into()))
    }
    async fn update_user(&self, _user: User) -> Result<User, Error> {
        Err(Error::Message("Not implemented".into()))
    }

    async fn list_users(&self, _start_id: Option<usize>, _page_size: Option<usize>) -> Result<Vec<User>, Error> {
        Err(Error::Message("Not implemented".into()))
    }

    async fn delete_user(&self, _id: i64) -> Result<User, Error> {
        Err(Error::Message("Not implemented".into()))
    }

    async fn find_by_id(&self, _id: i64) -> Result<Option<User>, Error> {
        Err(Error::Message("Not implemented".into()))
    }

    // async fn sample_work(&self) -> Result<(), Error> {
    //     sample_database_work(&self.repository_service).await
    // }
}
