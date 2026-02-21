//! Test utilities for mocking core services.
//! Only compiled when the `test-support` feature is enabled.

use std::sync::Arc;

use crate::CoreServices;
pub use crate::{session::MockSessionService, user::MockUserService};

/// Creates a CoreServices instance with the given mock UserService.
///
/// This is a convenience function for tests that need a CoreServices.
pub fn create_core_services_with_mock(mock: MockUserService) -> CoreServices {
    CoreServices {
        user_service: Arc::new(mock),
        session_service: Arc::new(MockSessionService::default()),
    }
}

/// Creates an Arc-wrapped CoreServices instance with the given mock
/// UserService.
///
/// This is a convenience function for tests that need an Arc<CoreServices>.
pub fn create_arc_core_services_with_mock(mock: MockUserService) -> Arc<CoreServices> {
    Arc::new(create_core_services_with_mock(mock))
}
