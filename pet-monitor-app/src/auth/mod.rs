//! This module provides utilities for authentication and verification.
//!
//! It contains the [`Token`] struct, which can be parsed to and from a string
//! and represents a JWT, and the [`validate()`] function, which verifies a
//! password against a hash.

use crate::secrets;
use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The claims in a JWT issued by this server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Claims {
    /// Issued-at time (Unix timestamp)
    iat: u64,
    /// Expiration time (Unix timestamp)
    exp: u64,
}

impl Claims {
    /// Creates new JWT claims that expire in `expires_in` time.
    fn new(expires_in: Duration) -> Self {
        let utc = Utc::now();
        Self {
            iat: utc.timestamp() as u64,
            exp: (utc + expires_in).timestamp() as u64,
        }
    }
}

impl Default for Claims {
    /// Creates new JWT claims that expire in 1 day.
    fn default() -> Self {
        Self::new(Duration::days(1))
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
    /// ```rust
    /// use pet_monitor_app::auth::Token;
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
    /// ```rust
    /// use chrono::Duration;
    /// use std::{thread, time};
    /// use pet_monitor_app::auth::Token;
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
    /// ```rust
    /// use pet_monitor_app::auth::Token;
    ///
    /// let token = Token::new();
    /// assert!(token.verify());
    /// ```
    pub fn verify(&self) -> bool {
        let utc = Utc::now();
        let exp = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.claims.exp.try_into().unwrap(), 0),
            Utc,
        );

        utc < exp
    }

    /// Parses a JWT into a `Token`.
    ///
    /// # Example
    /// ```rust
    /// use pet_monitor_app::{secrets, auth::Token};
    /// # fn main() -> Result<(), impl std::error::Error> {
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
        let dec_key = jwt::DecodingKey::from_secret(&**secret);

        match jwt::decode::<Claims>(s, &dec_key, &jwt::Validation::new(ALG)) {
            Ok(t) => Ok(Self {
                header: t.header,
                claims: t.claims,
            }),
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    /// Creates a JWT from a `Token`.
    ///
    /// # Example
    /// ```rust
    /// use pet_monitor_app::{secrets, auth::Token};
    /// # fn main() -> Result<(), impl std::error::Error> {
    ///
    /// let secret = secrets::Secret([0u8; 32]);
    ///
    /// let token = Token::new();
    /// let str_token = token.to_string(&secret)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_string(&self, secret: &secrets::Secret) -> jwt::errors::Result<String> {
        let enc_key = jwt::EncodingKey::from_secret(&**secret);

        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

/// Validates a password against a hash.
///
/// # Example
/// ```rust
/// use pet_monitor_app::{secrets, auth};
/// # fn main() -> Result<(), impl std::error::Error> {
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
    // unwrap should be safe if main has run
    argon2::verify_encoded(&hash, password.as_bytes())
}
