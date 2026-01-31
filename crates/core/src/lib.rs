pub mod error;
pub mod models;
pub mod services;
pub mod use_cases;

use std::sync::Arc;

pub use error::{Error, ErrorKind, RepositoryError};

use crate::services::{CoreServices, RepositoryService};

pub fn create_services(repository_service: Arc<RepositoryService>) -> Result<Arc<CoreServices>, Error> {
    let core_services = CoreServices::new(repository_service);

    Ok(Arc::new(core_services))
}
