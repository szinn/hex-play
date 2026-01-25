pub mod error;
pub mod models;
pub mod services;

use std::sync::Arc;

pub use error::*;
pub use models::*;
pub use services::*;

pub fn create_services(repository_service: Arc<RepositoryService>) -> Result<Arc<CoreServices>, Error> {
    let core_services = CoreServices::new(repository_service);

    Ok(Arc::new(core_services))
}
