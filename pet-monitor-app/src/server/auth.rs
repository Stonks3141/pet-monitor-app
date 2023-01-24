use super::AppState;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, Method, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken as jwt;
use jwt::errors::{ErrorKind, Result as JwtResult};
use jwt::get_current_timestamp as timestamp;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Claims {
    iat: u64,
    exp: u64,
}

impl Claims {
    fn new(expires_in: Duration) -> Self {
        let time = timestamp();
        Self {
            iat: time,
            exp: (time + expires_in.as_secs()) as u64,
        }
    }
}

const ALG: jwt::Algorithm = jwt::Algorithm::HS256;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    pub fn new(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }

    pub fn verify(&self) -> bool {
        timestamp() < self.claims.exp
    }

    pub fn decode(s: &str, secret: &[u8; 32]) -> JwtResult<Self> {
        let dec_key = jwt::DecodingKey::from_secret(secret); // TODO: only initialize this once
        let val = jwt::Validation::new(ALG);

        jwt::decode::<Claims>(s, &dec_key, &val).map(|t| Self {
            header: t.header,
            claims: t.claims,
        })
    }

    pub fn encode(&self, secret: &[u8; 32]) -> JwtResult<String> {
        let enc_key = jwt::EncodingKey::from_secret(secret);
        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    MissingToken,
    BadToken,
    InvalidToken,
    MissingCsrf,
    BadCsrf,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token"),
            Self::BadToken => (StatusCode::UNAUTHORIZED, "Bad token"),
            Self::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            Self::MissingCsrf => (StatusCode::UNAUTHORIZED, "Missing CSRF token"),
            Self::BadCsrf => (StatusCode::UNAUTHORIZED, "Bad CSRF token"),
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
        // TODO: use Iterator::try_find for this once it's stabilized

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
            Method::POST | Method::PUT | Method::DELETE | Method::CONNECT | Method::PATCH
                if parts
                    .headers
                    .get("x-csrf-token")
                    .ok_or(AuthError::MissingCsrf)?
                    .to_str()
                    .map_err(|_| AuthError::BadCsrf)?
                    != token =>
            {
                return Err(AuthError::BadCsrf);
            }
            _ => (),
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
