use std::sync::Arc;

use crate::RepositoryService;

pub struct CoreServices {
    pub(crate) repository_service: Arc<RepositoryService>,
}

impl CoreServices {
    #[tracing::instrument(level = "trace", skip(repository_service))]
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self { repository_service }
    }
}
