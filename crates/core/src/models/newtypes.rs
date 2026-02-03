//! Newtype wrappers for domain validation.
//!
//! These types provide compile-time guarantees about valid values.

use std::fmt;

use crate::Error;

/// A validated email address that must contain '@'.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Creates a new Email if the value contains '@'.
    ///
    /// # Errors
    ///
    /// Returns `Error::Validation` if the email doesn't contain '@'.
    pub fn new(email: impl Into<String>) -> Result<Self, Error> {
        let email = email.into();
        if !email.contains('@') {
            return Err(Error::Validation(format!("Invalid email format: {}", email)));
        }
        Ok(Self(email))
    }

    /// Returns the email as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Age with range validation (0-150).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Age(i16);

impl Age {
    /// Minimum valid age.
    pub const MIN: i16 = 0;
    /// Maximum valid age.
    pub const MAX: i16 = 150;

    /// Creates a new Age if the value is within valid range.
    ///
    /// # Errors
    ///
    /// Returns `Error::Validation` if age is outside 0-150 range.
    pub fn new(age: i16) -> Result<Self, Error> {
        if !(Self::MIN..=Self::MAX).contains(&age) {
            return Err(Error::Validation(format!("Age must be between {} and {}, got {}", Self::MIN, Self::MAX, age)));
        }
        Ok(Self(age))
    }

    /// Returns the age value.
    pub fn value(&self) -> i16 {
        self.0
    }
}

impl fmt::Display for Age {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Age> for i16 {
    fn from(age: Age) -> Self {
        age.0
    }
}

/// User ID that must be non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(i64);

impl UserId {
    /// Creates a new UserId if the value is non-negative.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidId` if the id is negative.
    pub fn new(id: i64) -> Result<Self, Error> {
        if id < 0 {
            return Err(Error::InvalidId(id));
        }
        Ok(Self(id))
    }

    /// Returns the user id value.
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<UserId> for i64 {
    fn from(id: UserId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================
    // Email tests
    // ==================
    #[test]
    fn test_email_valid() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_email_invalid_no_at() {
        let result = Email::new("invalid-email");
        assert!(result.is_err());
    }

    #[test]
    fn test_email_display() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(format!("{}", email), "test@example.com");
    }

    #[test]
    fn test_email_into_inner() {
        let email = Email::new("test@example.com").unwrap();
        let inner: String = email.into_inner();
        assert_eq!(inner, "test@example.com");
    }

    // ==================
    // Age tests
    // ==================
    #[test]
    fn test_age_valid() {
        let age = Age::new(25).unwrap();
        assert_eq!(age.value(), 25);
    }

    #[test]
    fn test_age_zero_valid() {
        let age = Age::new(0).unwrap();
        assert_eq!(age.value(), 0);
    }

    #[test]
    fn test_age_max_valid() {
        let age = Age::new(150).unwrap();
        assert_eq!(age.value(), 150);
    }

    #[test]
    fn test_age_negative_invalid() {
        let result = Age::new(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_age_over_max_invalid() {
        let result = Age::new(151);
        assert!(result.is_err());
    }

    #[test]
    fn test_age_default() {
        let age = Age::default();
        assert_eq!(age.value(), 0);
    }

    #[test]
    fn test_age_into_i16() {
        let age = Age::new(25).unwrap();
        let value: i16 = age.into();
        assert_eq!(value, 25);
    }

    // ==================
    // UserId tests
    // ==================
    #[test]
    fn test_user_id_valid() {
        let id = UserId::new(1).unwrap();
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn test_user_id_zero_valid() {
        let id = UserId::new(0).unwrap();
        assert_eq!(id.value(), 0);
    }

    #[test]
    fn test_user_id_negative_invalid() {
        let result = UserId::new(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_id_into_i64() {
        let id = UserId::new(42).unwrap();
        let value: i64 = id.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::new(123).unwrap();
        assert_eq!(format!("{}", id), "123");
    }
}
