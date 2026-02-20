//! Newtype wrappers for domain validation.
//!
//! These types provide compile-time guarantees about valid values.

use std::fmt;

use serde::{Deserialize, Serialize, de};

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

impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Email::new(s).map_err(|e| de::Error::custom(e.to_string()))
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

impl Serialize for Age {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i16(self.0)
    }
}

impl<'de> Deserialize<'de> for Age {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i16::deserialize(deserializer)?;
        Age::new(value).map_err(|e| de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

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
    // Email serde tests
    // ==================
    #[test]
    fn test_email_serialize() {
        let email = Email::new("test@example.com").unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, r#""test@example.com""#);
    }

    #[test]
    fn test_email_deserialize_valid() {
        let email: Email = serde_json::from_str(r#""test@example.com""#).unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_email_deserialize_invalid() {
        let result: Result<Email, _> = serde_json::from_str(r#""invalid-email""#);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid email format"));
    }

    #[test]
    fn test_email_roundtrip() {
        let original = Email::new("user@domain.com").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Email = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    // ==================
    // Age serde tests
    // ==================
    #[test]
    fn test_age_serialize() {
        let age = Age::new(25).unwrap();
        let json = serde_json::to_string(&age).unwrap();
        assert_eq!(json, "25");
    }

    #[test]
    fn test_age_deserialize_valid() {
        let age: Age = serde_json::from_str("30").unwrap();
        assert_eq!(age.value(), 30);
    }

    #[test]
    fn test_age_deserialize_invalid_negative() {
        let result: Result<Age, _> = serde_json::from_str("-5");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Age must be between"));
    }

    #[test]
    fn test_age_deserialize_invalid_over_max() {
        let result: Result<Age, _> = serde_json::from_str("200");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Age must be between"));
    }

    #[test]
    fn test_age_roundtrip() {
        let original = Age::new(42).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Age = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}
