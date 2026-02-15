use chrono::{DateTime, Utc};
use derive_builder::Builder;
use hex_play_utils::{define_token_prefix, token::Token};

use super::newtypes::{Age, Email};
use crate::Error;

define_token_prefix!(UserPrefix, "U_");
pub type UserId = u64;
pub type UserToken = Token<UserPrefix, UserId, { i64::MAX as u128 }>;

#[derive(Debug, Clone, Builder)]
pub struct User {
    #[builder(default = "0")]
    pub id: UserId,
    #[builder(default = "0")]
    pub version: u64,
    #[builder(default = "UserToken::generate()")]
    pub token: UserToken,
    pub name: String,
    pub email: Email,
    #[builder(default)]
    pub age: Age,
    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 0,
            version: 0,
            token: UserToken::generate(),
            name: String::new(),
            email: Email::new("default@example.com").expect("default email is valid"),
            age: Age::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl User {
    /// Creates a fake user with default timestamps and a generated token.
    /// Only available in test builds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn fake(id: UserId, name: impl Into<String>, email: impl Into<String>) -> Self {
        UserBuilder::default()
            .id(id)
            .version(0)
            .token(UserToken::new(id))
            .name(name.into())
            .email(Email::new(email).expect("test email should be valid"))
            .build()
            .expect("test user should build successfully")
    }

    /// Creates a fake user with a specific age, default timestamps and a
    /// generated token. Only available in test builds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn fake_with_age(id: UserId, name: impl Into<String>, email: impl Into<String>, age: i16) -> Self {
        UserBuilder::default()
            .id(id)
            .version(0)
            .token(UserToken::new(id))
            .name(name.into())
            .email(Email::new(email).expect("test email should be valid"))
            .age(Age::new(age).expect("test age should be valid"))
            .build()
            .expect("test user should build successfully")
    }
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub name: String,
    pub email: Email,
    pub age: Age,
}

impl NewUser {
    /// Creates a new user with validated email and age.
    ///
    /// # Errors
    ///
    /// Returns `Error::Validation` if email or age is invalid.
    pub fn new(name: impl Into<String>, email: impl Into<String>, age: i16) -> Result<Self, Error> {
        Ok(Self {
            name: name.into(),
            email: Email::new(email)?,
            age: Age::new(age)?,
        })
    }
}

impl Default for NewUser {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: Email::new("default@example.com").expect("default email is valid"),
            age: Age::default(),
        }
    }
}

/// Represents a partial update to a User.
///
/// Used to consolidate update logic between HTTP and gRPC handlers.
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Default, Clone)]
pub struct PartialUserUpdate {
    pub name: Option<String>,
    pub email: Option<Email>,
    pub age: Option<Age>,
}

impl PartialUserUpdate {
    /// Creates a new partial update with validated email and age if provided.
    ///
    /// # Errors
    ///
    /// Returns `Error::Validation` if email or age is invalid.
    pub fn new(name: Option<String>, email: Option<String>, age: Option<i16>) -> Result<Self, Error> {
        Ok(Self {
            name,
            email: email.map(Email::new).transpose()?,
            age: age.map(Age::new).transpose()?,
        })
    }

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
