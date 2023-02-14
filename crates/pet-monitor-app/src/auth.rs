use crate::AppState;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use hyper::Request;
use jsonwebtoken as jwt;
use jwt::errors::{ErrorKind, Result as JwtResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower_cookies::Cookie;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub iat: u64,
    pub exp: u64,
}

impl Claims {
    pub fn new(expires_in: Duration) -> Self {
        let time = jwt::get_current_timestamp();
        Self {
            iat: time,
            exp: time + expires_in.as_secs(),
        }
    }
}

const ALG: jwt::Algorithm = jwt::Algorithm::HS256;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    header: jwt::Header,
    pub claims: Claims,
}

impl Token {
    pub fn new(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }

    pub fn verify(&self) -> bool {
        jwt::get_current_timestamp() < self.claims.exp
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
            Self::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token"),
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
        // TODO: use Iterator::try_find for this once it's stabilized

        let cookies: Vec<_> = parts
            .headers
            .get(header::COOKIE)
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::MissingToken)?
            .split("; ")
            .map(Cookie::parse)
            .collect::<Result<_, _>>()
            .map_err(|_| AuthError::MissingToken)?;

        let token = cookies
            .iter()
            .find(|cookie| cookie.name() == "token")
            .ok_or(AuthError::MissingToken)?
            .value();

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

pub async fn auth_error_layer<B>(req: Request<B>, next: Next<B>) -> Response {
    let res = next.run(req).await;
    if res.status() == StatusCode::UNAUTHORIZED {
        Redirect::to("/login.html").into_response()
    } else {
        res
    }
}
