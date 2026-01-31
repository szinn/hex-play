use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[derive(Debug, Default, Clone, Builder)]
pub struct User {
    #[builder(default = "0")]
    pub id: i64,
    #[builder(default = "0")]
    pub version: i64,
    pub name: String,
    pub email: String,
    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a test user with default timestamps.
    /// Only available in test builds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn test(id: i64, name: impl Into<String>, email: impl Into<String>) -> Self {
        UserBuilder::default()
            .id(id)
            .version(0)
            .name(name.into())
            .email(email.into())
            .build()
            .expect("test user should build successfully")
    }
}

#[derive(Debug, Default, Builder)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}
