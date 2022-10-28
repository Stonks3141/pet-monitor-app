//! This module provides utilities for authentication and verification.
//!
//! It contains the [`Token`] struct, which can be parsed to and from a string
//! and represents a JWT, and the [`validate`] function, which verifies a
//! password against a hash.

use super::provider::Provider;
use crate::config::Context;
use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use jwt::errors::{Error, ErrorKind, Result};
use log::warn;
use rocket::http::{Cookie, Method, Status};
use rocket::request::{FromRequest, Outcome, Request};
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

/// The algorithm used for signing JWTs.
const ALG: jwt::Algorithm = jwt::Algorithm::HS256;

/// A struct representing a JWT.
#[derive(Debug, PartialEq)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    /// Creates a new token that expires in `expires_in` time.
    pub fn new(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }

    /// Verifies the validity of a `Token`.
    pub fn verify(&self) -> bool {
        let utc = Utc::now();
        let exp = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.claims.exp as i64, 0),
            Utc,
        );

        utc < exp
    }

    /// Parses a JWT into a `Token`.
    pub fn from_str(s: &str, secret: &[u8; 32]) -> Result<Self> {
        let dec_key = jwt::DecodingKey::from_secret(secret);
        let val = jwt::Validation::new(ALG);

        jwt::decode::<Claims>(s, &dec_key, &val).map(|t| Self {
            header: t.header,
            claims: t.claims,
        })
    }

    /// Creates a JWT from a `Token`.
    pub fn to_string(&self, secret: &[u8; 32]) -> Result<String> {
        let enc_key = jwt::EncodingKey::from_secret(secret);

        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

/// This request guard extracts a token from a request. It also checks the
/// validity of the token and checks the "x-csrf-token" header for
/// state-changing request methods.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = Error;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(token) = req.cookies().get("token").map(Cookie::value) {
            match req.method() {
                Method::Get | Method::Head | Method::Options | Method::Trace => (),
                _ => {
                    if let Some(csrf_token) = req.headers().get_one("x-csrf-token") {
                        if token != csrf_token {
                            return Outcome::Failure((
                                Status::Unauthorized,
                                Error::from(ErrorKind::InvalidToken),
                            ));
                        }
                    } else {
                        return Outcome::Failure((
                            Status::Unauthorized,
                            Error::from(ErrorKind::InvalidToken),
                        ));
                    }
                }
            }
            let ctx = match req.rocket().state::<Provider<Context>>() {
                Some(v) => v,
                None => {
                    warn!("Rocket is not managing a `Provider<Context>`");
                    return Outcome::Failure((
                        Status::InternalServerError,
                        Error::from(ErrorKind::InvalidToken),
                    ));
                }
            }
            .get();

            match Self::from_str(token, &ctx.jwt_secret) {
                Ok(token) => {
                    if token.verify() {
                        Outcome::Success(token)
                    } else {
                        Outcome::Failure((
                            Status::Unauthorized,
                            Error::from(ErrorKind::InvalidToken),
                        ))
                    }
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Base64(e) => warn!("Parsing JWT failed with error {:?}", e),
                        ErrorKind::Crypto(e) => warn!("Parsing JWT failed with error {:?}", e),
                        ErrorKind::Json(e) => warn!("Parsing JWT failed with error {:?}", e),
                        ErrorKind::Utf8(e) => warn!("Parsing JWT failed with error {:?}", e),
                        _ => (),
                    }
                    match e.kind() {
                        ErrorKind::Base64(_)
                        | ErrorKind::Crypto(_)
                        | ErrorKind::Json(_)
                        | ErrorKind::Utf8(_) => Outcome::Failure((Status::InternalServerError, e)),
                        _ => Outcome::Failure((Status::Unauthorized, e)),
                    }
                }
            }
        } else {
            Outcome::Failure((Status::Unauthorized, Error::from(ErrorKind::InvalidToken)))
        }
    }
}