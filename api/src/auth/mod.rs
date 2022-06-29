// Pet Montitor App
// Copyright (C) 2022  Samuel Nystrom
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
    /// use pet_monitor_app::{secrets, auth::Token};
    ///
    /// let secret = secrets::Secret([0u8; 32]);
    ///
    /// let token = Token::new();
    /// let str_token = token.to_string(&secret).unwrap();
    /// let new_token = Token::from_str(&str_token, &secret).unwrap();
    /// assert_eq!(token, new_token);
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
    /// ```
    /// use pet_monitor_app::{secrets, auth::Token};
    ///
    /// let secret = secrets::Secret([0u8; 32]);
    ///
    /// let token = Token::new();
    /// let str_token = token.to_string(&secret).unwrap();
    /// ```
    pub fn to_string(&self, secret: &secrets::Secret) -> jwt::errors::Result<String> {
        let enc_key = jwt::EncodingKey::from_secret(&**secret);

        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

/// Validates a password against a hash.
///
/// # Example
/// ```
/// use pet_monitor_app::{secrets, auth};
///
/// let password = "password";
/// let config = argon2::Config::default();
/// let hash = secrets::Password(
///     argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap());
///
/// assert!(auth::validate(password, &hash).unwrap());
/// ```
pub fn validate(password: &str, hash: &secrets::Password) -> argon2::Result<bool> {
    // unwrap should be safe if main has run
    argon2::verify_encoded(&hash, password.as_bytes())
}
