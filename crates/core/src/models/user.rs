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

/// Represents a partial update to a User.
///
/// Used to consolidate update logic between HTTP and gRPC handlers.
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Default, Clone)]
pub struct PartialUserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i16>,
}

impl PartialUserUpdate {
    /// Apply this partial update to an existing user.
    ///
    /// Only modifies fields that have `Some` values.
    pub fn apply_to(&self, user: &mut User) {
        if let Some(name) = &self.name {
            user.name = name.clone();
        }
        if let Some(email) = &self.email {
            user.email = email.clone();
        }
        if let Some(age) = self.age {
            user.age = age;
        }
    }

    /// Returns true if all fields are None.
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.email.is_none() && self.age.is_none()
    }
}
