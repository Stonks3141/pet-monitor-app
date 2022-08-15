//! This module provides utilities for authentication and verification.
//!
//! It contains the [`Token`] struct, which can be parsed to and from a string
//! and represents a JWT, and the [`validate()`] function, which verifies a
//! password against a hash.

use crate::secrets;
//use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use rocket::serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(test)]
mod tests;

/// The claims in a JWT issued by this server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Claims {
    /// Issued-at time (Unix timestamp)
    iat: u64,
    /// Expiration time (Unix timestamp)
    exp: u64,
}

impl Claims {
    /// Creates new JWT claims that expire in `expires_in` time.
    fn new(expires_in: Duration) -> Self {
        let now = jwt::get_current_timestamp();
        Self {
            iat: now,
            exp: now + expires_in.as_secs(),
        }
    }
}

impl Default for Claims {
    /// Creates new JWT claims that expire in 1 day.
    fn default() -> Self {
        Self::new(Duration::from_secs(60 * 60 * 24))
    }
}

/// The algorithm used for signing JWTs.
const ALG: jwt::Algorithm = jwt::Algorithm::HS256;

/// A struct representing a JWT.
#[derive(Debug, PartialEq)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    /// Creates a new token that expires in 1 day.
    ///
    /// # Example
    /// ```no_test
    /// use crate::auth::Token;
    ///
    /// let token = Token::new();
    /// assert!(token.verify());
    /// ```
    pub fn new() -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::default(),
        }
    }

    /// Creates a new token that expires in `expires_in` time.
    ///
    /// # Example
    /// ```no_test
    /// use chrono::Duration;
    /// use std::{thread, time};
    /// use crate::auth::Token;
    ///
    /// let token = Token::with_expiration(Duration::seconds(1));
    /// thread::sleep(time::Duration::from_secs(2));
    /// assert!(!token.verify());
    /// ```
    pub fn with_expiration(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }

    /// Verifies the validity of a `Token`.
    ///
    /// # Example
    /// ```no_test
    /// use crate::auth::Token;
    ///
    /// let token = Token::new();
    /// assert!(token.verify());
    /// ```
    pub fn verify(&self) -> bool {
        let now = jwt::get_current_timestamp();

        now < self.claims.exp
    }

    /// Parses a JWT into a `Token`.
    ///
    /// # Example
    /// ```no_test
    /// use crate::{secrets, auth::Token};
    /// # fn main() -> jsonwebtoken::errors::Result<()> {
    ///
    /// let secret = secrets::Secret([0u8; 32]);
    ///
    /// let token = Token::new();
    /// let str_token = token.to_string(&secret)?;
    /// let new_token = Token::from_str(&str_token, &secret)?;
    /// assert_eq!(token, new_token);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_str(s: &str, secret: &secrets::Secret) -> jwt::errors::Result<Self> {
        let dec_key = jwt::DecodingKey::from_secret(&secret.0);
        let val = jwt::Validation::new(ALG);

        jwt::decode::<Claims>(s, &dec_key, &val).map(|t| Self {
            header: t.header,
            claims: t.claims,
        })
    }

    /// Creates a JWT from a `Token`.
    ///
    /// # Example
    /// ```no_test
    /// use crate::{secrets, auth::Token};
    /// # fn main() -> jsonwebtoken::errors::Result<()> {
    ///
    /// let secret = secrets::Secret([0u8; 32]);
    ///
    /// let token = Token::new();
    /// let str_token = token.to_string(&secret)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_string(&self, secret: &secrets::Secret) -> jwt::errors::Result<String> {
        let enc_key = jwt::EncodingKey::from_secret(&secret.0);

        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates a password against a hash.
///
/// # Example
/// ```no_test
/// use crate::{secrets, auth};
/// # fn main() -> jsonwebtoken::errors::Result<()> {
///
/// let password = "password";
/// let config = argon2::Config::default();
/// let hash = secrets::Password(
///     argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap());
///
/// assert!(auth::validate(password, &hash).unwrap());
/// # Ok(())
/// # }
/// ```
pub fn validate(password: &str, hash: &secrets::Password) -> argon2::Result<bool> {
    argon2::verify_encoded(&hash.0, password.as_bytes())
}
