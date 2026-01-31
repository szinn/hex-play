use std::sync::Arc;

use crate::{
    repositories::RepositoryService,
    use_cases::{UserUseCases, UserUseCasesImpl},
};

pub struct CoreServices {
    pub user: Arc<dyn UserUseCases>,
}

impl CoreServices {
    #[tracing::instrument(level = "trace", skip(repository_service))]
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self {
            user: Arc::new(UserUseCasesImpl::new(repository_service)),
        }
    }
}
