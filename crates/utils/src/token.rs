use std::{fmt, hash::Hash, marker::PhantomData, str::FromStr};

use serde::{Deserialize, Serialize, de};
use thiserror::Error;

/// Base-32 alphabet excluding visually ambiguous characters (I, L, O, Q).
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKMNPRSTUVWXYZ0123456789";

/// Length of the encoded identifier portion of a token.
const ENCODED_LEN: usize = 13;

/// Trait that defines the prefix for a token kind.
pub trait TokenPrefix: fmt::Debug + Clone + PartialEq + Eq {
    const PREFIX: &'static str;
}

/// A typed, prefixed identifier for domain entities.
///
/// Stores only a `u64` internally. The string representation (e.g.
/// `U_ABCD1234NRST0`) is computed on demand via [`Display`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token<P: TokenPrefix> {
    id: u64,
    _marker: PhantomData<P>,
}

impl<P: TokenPrefix> fmt::Debug for Token<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({self})")
    }
}

/// Errors that can occur when parsing a token from a string.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenError {
    #[error("invalid prefix: expected \"{expected}\", found \"{found}\"")]
    InvalidPrefix { expected: &'static str, found: String },

    #[error("invalid length: expected {expected}, found {found}")]
    InvalidLength { expected: usize, found: usize },

    #[error("invalid character: '{0}'")]
    InvalidCharacter(char),

    #[error("encoded value overflows u64")]
    Overflow,
}

/// Encode a `u64` into a 13-character base-32 string using [`ALPHABET`].
fn encode(value: u64) -> String {
    let mut buf = [b'A'; ENCODED_LEN];
    let mut remaining = value;
    for i in (0..ENCODED_LEN).rev() {
        buf[i] = ALPHABET[(remaining & 0x1F) as usize];
        remaining >>= 5;
    }
    // SAFETY: all bytes come from ALPHABET which is ASCII
    String::from_utf8(buf.to_vec()).expect("alphabet is ASCII")
}

/// Decode a 13-character base-32 string back to a `u64`.
fn decode(s: &str) -> Result<u64, TokenError> {
    let mut value: u64 = 0;
    for ch in s.chars() {
        let idx = ALPHABET.iter().position(|&c| c == ch as u8).ok_or(TokenError::InvalidCharacter(ch))?;
        value = value.checked_shl(5).and_then(|v| v.checked_add(idx as u64)).ok_or(TokenError::Overflow)?;
    }
    Ok(value)
}

impl<P: TokenPrefix> Token<P> {
    /// Create a token from a numeric ID.
    pub fn new(id: u64) -> Self {
        Self { id, _marker: PhantomData }
    }

    /// Parse a token from its string representation (e.g. `"U_ABCD1234NRST0"`).
    pub fn parse(s: &str) -> Result<Self, TokenError> {
        let prefix = P::PREFIX;
        if !s.starts_with(prefix) {
            let found_len = s.len().min(prefix.len());
            return Err(TokenError::InvalidPrefix {
                expected: prefix,
                found: s[..found_len].to_string(),
            });
        }

        let encoded = &s[prefix.len()..];
        if encoded.len() != ENCODED_LEN {
            return Err(TokenError::InvalidLength {
                expected: prefix.len() + ENCODED_LEN,
                found: s.len(),
            });
        }

        let id = decode(encoded)?;
        Ok(Self::new(id))
    }

    /// Get the underlying numeric ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Check if a string is a well-formed token of this type.
    pub fn is_valid(s: &str) -> bool {
        Self::parse(s).is_ok()
    }
}

impl<P: TokenPrefix> fmt::Display for Token<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", P::PREFIX, encode(self.id))
    }
}

impl<P: TokenPrefix> FromStr for Token<P> {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<P: TokenPrefix> Serialize for Token<P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, P: TokenPrefix> Deserialize<'de> for Token<P> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(de::Error::custom)
    }
}

/// Define a token prefix type and its associated `TokenPrefix` implementation.
///
/// # Example
///
/// ```
/// use hex_play_utils::{define_token_prefix, token::Token};
///
/// define_token_prefix!(UserPrefix, "U_");
/// type UserToken = Token<UserPrefix>;
/// ```
#[macro_export]
macro_rules! define_token_prefix {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;

        impl $crate::token::TokenPrefix for $name {
            const PREFIX: &'static str = $prefix;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    define_token_prefix!(TestPrefix, "T_");
    type TestToken = Token<TestPrefix>;

    define_token_prefix!(UserPrefix, "U_");
    type UserToken = Token<UserPrefix>;

    #[test]
    fn round_trip() {
        for id in [0, 1, 42, 1000, 123_456_789, u64::MAX] {
            let token = TestToken::new(id);
            let s = token.to_string();
            let parsed = TestToken::parse(&s).unwrap();
            assert_eq!(parsed.id(), id);
        }
    }

    #[test]
    fn zero_encodes_to_all_first_char() {
        let token = TestToken::new(0);
        assert_eq!(token.to_string(), "T_AAAAAAAAAAAAA");
    }

    #[test]
    fn u64_max_round_trips() {
        let token = TestToken::new(u64::MAX);
        let s = token.to_string();
        let parsed = TestToken::parse(&s).unwrap();
        assert_eq!(parsed.id(), u64::MAX);
    }

    #[test]
    fn known_value_encoding() {
        let token = TestToken::new(1);
        let s = token.to_string();
        assert_eq!(s, "T_AAAAAAAAAAAAB");
    }

    #[test]
    fn wrong_prefix_error() {
        let err = UserToken::parse("T_AAAAAAAAAAAAA").unwrap_err();
        assert_eq!(
            err,
            TokenError::InvalidPrefix {
                expected: "U_",
                found: "T_".to_string(),
            }
        );
    }

    #[test]
    fn invalid_character_error() {
        // 'I' is not in the alphabet
        let err = TestToken::parse("T_AAAAAAAAAAAIA").unwrap_err();
        assert_eq!(err, TokenError::InvalidCharacter('I'));
    }

    #[test]
    fn excluded_characters_rejected() {
        for ch in ['I', 'L', 'O', 'Q'] {
            let s = format!("T_AAAAAAAAAAAA{ch}");
            let err = TestToken::parse(&s).unwrap_err();
            assert_eq!(err, TokenError::InvalidCharacter(ch));
        }
    }

    #[test]
    fn wrong_length_error() {
        let err = TestToken::parse("T_AAAA").unwrap_err();
        assert_eq!(err, TokenError::InvalidLength { expected: 15, found: 6 });
    }

    #[test]
    fn is_valid_returns_true_for_valid() {
        let s = TestToken::new(42).to_string();
        assert!(TestToken::is_valid(&s));
    }

    #[test]
    fn is_valid_returns_false_for_invalid() {
        assert!(!TestToken::is_valid("INVALID"));
        assert!(!TestToken::is_valid("T_SHORT"));
        assert!(!TestToken::is_valid("X_AAAAAAAAAAAAA"));
    }

    #[test]
    fn from_str_works() {
        let s = TestToken::new(99).to_string();
        let parsed: TestToken = s.parse().unwrap();
        assert_eq!(parsed.id(), 99);
    }

    #[test]
    fn different_prefix_types_are_distinct() {
        let test_s = TestToken::new(42).to_string();
        let user_s = UserToken::new(42).to_string();
        assert_ne!(test_s, user_s);
        assert!(test_s.starts_with("T_"));
        assert!(user_s.starts_with("U_"));
    }

    #[test]
    fn serde_round_trip() {
        let token = TestToken::new(123_456);
        let json = serde_json::to_string(&token).unwrap();
        let parsed: TestToken = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id(), 123_456);
    }

    #[test]
    fn serde_serializes_as_string() {
        let token = TestToken::new(0);
        let json = serde_json::to_string(&token).unwrap();
        assert_eq!(json, r#""T_AAAAAAAAAAAAA""#);
    }

    #[test]
    fn serde_rejects_invalid_token() {
        let result = serde_json::from_str::<TestToken>(r#""INVALID""#);
        assert!(result.is_err());
    }

    #[test]
    fn debug_format() {
        let token = TestToken::new(0);
        let debug = format!("{token:?}");
        assert_eq!(debug, "Token(T_AAAAAAAAAAAAA)");
    }
}
