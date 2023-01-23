use crate::server::AppState;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, Method, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use jwt::errors::{ErrorKind, Result as JwtResult};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, PartialEq, Eq)]
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
        let Some(naive_time) = NaiveDateTime::from_timestamp_opt(self.claims.exp as i64, 0) else {
            return false;
        };
        let exp = DateTime::<Utc>::from_utc(naive_time, Utc);

        utc < exp
    }

    /// Parses a JWT into a `Token`.
    pub fn decode(s: &str, secret: &[u8; 32]) -> JwtResult<Self> {
        let dec_key = jwt::DecodingKey::from_secret(secret); // TODO: only initialize this once
        let val = jwt::Validation::new(ALG);

        jwt::decode::<Claims>(s, &dec_key, &val).map(|t| Self {
            header: t.header,
            claims: t.claims,
        })
    }

    /// Creates a JWT from a `Token`.
    pub fn encode(&self, secret: &[u8; 32]) -> JwtResult<String> {
        let enc_key = jwt::EncodingKey::from_secret(secret);
        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    MissingToken,
    BadToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            Self::BadToken => (StatusCode::UNAUTHORIZED, "Bad token"),
            Self::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        }
        .into_response()
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Token {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies: Vec<_> = parts
            .headers
            .get(header::COOKIE)
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::MissingToken)?
            .split("; ")
            .map(|cookie| cookie.split_once('=').ok_or(AuthError::MissingToken))
            .collect::<Result<_, _>>()?;

        let token = cookies
            .into_iter()
            .find(|(key, _)| *key == "token")
            .ok_or(AuthError::MissingToken)?
            .1;

        match parts.method {
            Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE => (),
            _ => {
                if parts
                    .headers
                    .get("x-csrf-token")
                    .ok_or(AuthError::MissingToken)?
                    .to_str()
                    .map_err(|_| AuthError::InvalidToken)?
                    != token
                {
                    return Err(AuthError::BadToken);
                }
            }
        }

        match Token::decode(token, &state.ctx.get().jwt_secret) {
            Ok(token) => {
                if token.verify() {
                    Ok(token)
                } else {
                    Err(AuthError::BadToken)
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::Base64(_)
                | ErrorKind::Crypto(_)
                | ErrorKind::Json(_)
                | ErrorKind::Utf8(_) => Err(AuthError::InvalidToken),
                _ => Err(AuthError::BadToken),
            },
        }
    }
}
