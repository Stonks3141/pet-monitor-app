//! This module provides utilities for authentication and verification.
//!
//! It contains the [`Token`] struct, which can be parsed to and from a string
//! and represents a JWT, and the [`validate()`] function, which verifies a
//! password against a hash.

use crate::secrets;
use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(test)]
mod tests;

/// The claims in a JWT
#[derive(Debug, Serialize, Deserialize)]
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

/// A struct for issuing and verifying JWTs.
#[derive(Debug)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    /// Creates a new token that expires in 1 day.
    pub fn new() -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::default(),
        }
    }

    /// Creates a new token that expires in `expires_in` time.
    pub fn with_expiration(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }
}

impl FromStr for Token {
    type Err = jwt::errors::Error;

    /// Parses a JWT into a `Token`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dec_key = jwt::DecodingKey::from_secret(secrets::SECRET.get().unwrap());

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
}

impl TryFrom<Token> for String {
    type Error = jwt::errors::Error;

    /// Creates a JWT from a `Token`.
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let enc_key = jwt::EncodingKey::from_secret(secrets::SECRET.get().unwrap());

        jwt::encode(&token.header, &token.claims, &enc_key)
    }
}

impl Default for Token {
    /// An alias for `Token::new()`.
    fn default() -> Self {
        Self::new()
    }
}

/// Validates a password against [`secrets::PASSWORD_HASH`].
pub fn validate(password: &str) -> argon2::Result<bool> {
    // unwrap should be safe if main has run
    argon2::verify_encoded(secrets::PASSWORD_HASH.get().unwrap(), password.as_bytes())
}
