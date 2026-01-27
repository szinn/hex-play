use std::sync::Arc;

use crate::{Error, services::RepositoryService, use_cases::user::sample_database_work};

mod sample;

use sample::*;

pub(crate) struct UserUseCasesImpl {
    repository_service: Arc<RepositoryService>,
}

#[async_trait::async_trait]
pub trait UserUseCases: Send + Sync {
    async fn sample_work(&self) -> Result<(), Error>;
}

impl UserUseCasesImpl {
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self { repository_service }
    }
}

#[async_trait::async_trait]
impl UserUseCases for UserUseCasesImpl {
    async fn sample_work(&self) -> Result<(), Error> {
        sample_database_work(&self.repository_service).await
    }
}
