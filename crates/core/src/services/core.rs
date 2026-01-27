use std::sync::Arc;

use crate::{
    services::RepositoryService,
    use_cases::{UserUseCases, UserUseCasesImpl},
};

pub struct CoreServices {
    pub(crate) repository_service: Arc<RepositoryService>,
    pub user: Arc<dyn UserUseCases>,
}

impl CoreServices {
    #[tracing::instrument(level = "trace", skip(repository_service))]
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self {
            repository_service: repository_service.clone(),
            user: Arc::new(UserUseCasesImpl::new(repository_service.clone())),
        }
    }
}
