pub mod session;

pub mod user;

use std::sync::Arc;

pub use session::SessionService;
pub use user::UserService;
pub(crate) use user::UserServiceImpl;

use crate::{repositories::RepositoryService, services::session::SessionServiceImpl};

pub struct CoreServices {
    pub user_service: Arc<dyn UserService>,
    pub session_service: Arc<dyn SessionService>,
}

impl CoreServices {
    #[tracing::instrument(level = "trace", skip(repository_service))]
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self {
            user_service: Arc::new(UserServiceImpl::new(repository_service.clone())),
            session_service: Arc::new(SessionServiceImpl::new(repository_service)),
        }
    }
}
