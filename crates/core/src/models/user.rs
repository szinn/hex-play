use chrono::{DateTime, Utc};
use derive_builder::Builder;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Builder)]
pub struct User {
    #[builder(default = "0")]
    pub id: i64,
    #[builder(default = "0")]
    pub version: i64,
    #[builder(default = "Uuid::nil()")]
    pub token: Uuid,
    pub name: String,
    pub email: String,
    #[builder(default = "0")]
    pub age: i16,
    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a test user with default timestamps and a generated token.
    /// Only available in test builds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn test(id: i64, name: impl Into<String>, email: impl Into<String>) -> Self {
        UserBuilder::default()
            .id(id)
            .version(0)
            .token(Uuid::new_v4())
            .name(name.into())
            .email(email.into())
            .build()
            .expect("test user should build successfully")
    }

    /// Creates a test user with a specific age, default timestamps and a
    /// generated token. Only available in test builds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn test_with_age(id: i64, name: impl Into<String>, email: impl Into<String>, age: i16) -> Self {
        UserBuilder::default()
            .id(id)
            .version(0)
            .token(Uuid::new_v4())
            .name(name.into())
            .email(email.into())
            .age(age)
            .build()
            .expect("test user should build successfully")
    }
}

#[derive(Debug, Default, Clone, Builder)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    #[builder(default = "0")]
    pub age: i16,
}
