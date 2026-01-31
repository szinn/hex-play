pub mod user;

use std::sync::Arc;

pub use user::UserService;
pub(crate) use user::UserServiceImpl;

use crate::repositories::RepositoryService;

pub struct CoreServices {
    pub user_service: Arc<dyn UserService>,
}

impl CoreServices {
    #[tracing::instrument(level = "trace", skip(repository_service))]
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self {
            user_service: Arc::new(UserServiceImpl::new(repository_service)),
        }
    }
}
