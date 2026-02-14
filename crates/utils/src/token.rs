use std::{fmt, hash::Hash, marker::PhantomData, str::FromStr};

use rand::Rng as _;
use serde::{Deserialize, Serialize, de};
use thiserror::Error;

/// Base-32 alphabet excluding visually ambiguous characters (I, L, O, Q).
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKMNPRSTUVWXYZ0123456789";

/// Trait that defines the prefix for a token kind.
pub trait TokenPrefix: fmt::Debug + Clone + PartialEq + Eq {
    const PREFIX: &'static str;
}

/// Trait that abstracts encode/decode over the numeric backing type.
pub trait TokenId: Copy + PartialEq + Eq + Hash + fmt::Debug {
    /// Length of the encoded identifier portion (excluding the prefix).
    const ENCODED_LEN: usize;

    /// Encode this value into a base-32 string of [`Self::ENCODED_LEN`]
    /// characters.
    fn encode(self) -> String;

    /// Decode a base-32 string back into this numeric type.
    fn decode(s: &str) -> Result<Self, TokenError>;

    /// Generate a random value.
    fn random() -> Self;
}

impl TokenId for u64 {
    const ENCODED_LEN: usize = 13; // 32^13 > u64::MAX

    fn random() -> Self {
        rand::rng().random()
    }

    fn encode(self) -> String {
        let mut buf = [b'A'; 13];
        let mut remaining = self;
        for i in (0..13).rev() {
            buf[i] = ALPHABET[(remaining & 0x1F) as usize];
            remaining >>= 5;
        }
        // SAFETY: all bytes come from ALPHABET which is ASCII
        String::from_utf8(buf.to_vec()).expect("alphabet is ASCII")
    }

    fn decode(s: &str) -> Result<Self, TokenError> {
        let mut value: u64 = 0;
        for ch in s.chars() {
            let idx = ALPHABET.iter().position(|&c| c == ch as u8).ok_or(TokenError::InvalidCharacter(ch))?;
            value = value.checked_shl(5).and_then(|v| v.checked_add(idx as u64)).ok_or(TokenError::Overflow)?;
        }
        Ok(value)
    }
}

impl TokenId for u128 {
    const ENCODED_LEN: usize = 26; // 32^26 > u128::MAX

    fn random() -> Self {
        rand::rng().random()
    }

    fn encode(self) -> String {
        let mut buf = [b'A'; 26];
        let mut remaining = self;
        for i in (0..26).rev() {
            buf[i] = ALPHABET[(remaining & 0x1F) as usize];
            remaining >>= 5;
        }
        // SAFETY: all bytes come from ALPHABET which is ASCII
        String::from_utf8(buf.to_vec()).expect("alphabet is ASCII")
    }

    fn decode(s: &str) -> Result<Self, TokenError> {
        let mut value: u128 = 0;
        for ch in s.chars() {
            let idx = ALPHABET.iter().position(|&c| c == ch as u8).ok_or(TokenError::InvalidCharacter(ch))?;
            value = value.checked_shl(5).and_then(|v| v.checked_add(idx as u128)).ok_or(TokenError::Overflow)?;
        }
        Ok(value)
    }
}

/// A typed, prefixed identifier for domain entities.
///
/// Stores only the numeric ID internally. The string representation (e.g.
/// `U_ABCD1234NRST0`) is computed on demand via [`Display`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token<P: TokenPrefix, I: TokenId = u64> {
    id: I,
    _marker: PhantomData<P>,
}

impl<P: TokenPrefix, I: TokenId> fmt::Debug for Token<P, I> {
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

    #[error("encoded value overflow")]
    Overflow,
}

impl<P: TokenPrefix, I: TokenId> Token<P, I> {
    /// Create a token from a numeric ID.
    pub fn new(id: I) -> Self {
        Self { id, _marker: PhantomData }
    }

    /// Generate a new token with a random ID.
    pub fn generate() -> Self {
        Self::new(I::random())
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
        if encoded.len() != I::ENCODED_LEN {
            return Err(TokenError::InvalidLength {
                expected: prefix.len() + I::ENCODED_LEN,
                found: s.len(),
            });
        }

        let id = I::decode(encoded)?;
        Ok(Self::new(id))
    }

    /// Get the underlying numeric ID.
    pub fn id(&self) -> I {
        self.id
    }

    /// Check if a string is a well-formed token of this type.
    pub fn is_valid(s: &str) -> bool {
        Self::parse(s).is_ok()
    }
}

impl<P: TokenPrefix, I: TokenId> fmt::Display for Token<P, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", P::PREFIX, self.id.encode())
    }
}

impl<P: TokenPrefix, I: TokenId> FromStr for Token<P, I> {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<P: TokenPrefix, I: TokenId> Serialize for Token<P, I> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, P: TokenPrefix, I: TokenId> Deserialize<'de> for Token<P, I> {
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
/// type UserToken = Token<UserPrefix>;          // u64 (default)
///
/// define_token_prefix!(SessionPrefix, "S_");
/// type SessionToken = Token<SessionPrefix, u128>; // u128
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

    // --- u64 tests (unchanged, use default type parameter) ---

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

    // --- u128 tests ---

    define_token_prefix!(BigPrefix, "B_");
    type BigToken = Token<BigPrefix, u128>;

    #[test]
    fn u128_round_trip() {
        for id in [0u128, 1, u64::MAX as u128, u128::MAX] {
            let token = BigToken::new(id);
            let s = token.to_string();
            let parsed = BigToken::parse(&s).unwrap();
            assert_eq!(parsed.id(), id);
        }
    }

    #[test]
    fn u128_zero_encodes_to_26_as() {
        let token = BigToken::new(0);
        assert_eq!(token.to_string(), "B_AAAAAAAAAAAAAAAAAAAAAAAAAA");
    }

    #[test]
    fn u128_max_round_trips() {
        let token = BigToken::new(u128::MAX);
        let s = token.to_string();
        let parsed = BigToken::parse(&s).unwrap();
        assert_eq!(parsed.id(), u128::MAX);
    }

    #[test]
    fn u128_known_value_encoding() {
        let token = BigToken::new(1);
        let s = token.to_string();
        // 25 A's + B
        assert_eq!(s, "B_AAAAAAAAAAAAAAAAAAAAAAAAAB");
    }

    #[test]
    fn u128_wrong_length_error() {
        // prefix (2) + encoded (26) = 28
        let err = BigToken::parse("B_AAAA").unwrap_err();
        assert_eq!(err, TokenError::InvalidLength { expected: 28, found: 6 });
    }

    #[test]
    fn u128_serde_round_trip() {
        let token = BigToken::new(123_456);
        let json = serde_json::to_string(&token).unwrap();
        let parsed: BigToken = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id(), 123_456);
    }

    #[test]
    fn u128_debug_format() {
        let token = BigToken::new(0);
        let debug = format!("{token:?}");
        assert_eq!(debug, "Token(B_AAAAAAAAAAAAAAAAAAAAAAAAAA)");
    }
}
