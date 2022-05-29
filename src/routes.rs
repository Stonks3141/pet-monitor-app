use rocket::fs::{relative, NamedFile};
use rocket::get;

// match all for React Router
#[get("/<_f..>", rank = 2)]
pub async fn index(_f: std::path::PathBuf) -> Option<NamedFile> {
    NamedFile::open(relative!("client/build/index.html"))
        .await
        .ok()
}

pub mod auth {
    use chrono::prelude::*;
    use jsonwebtoken as jwt;
    use rocket::http::{Cookie, CookieJar, Status};
    use rocket::{post, Request, Response};
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        iat: String,
        exp: String,
    }

    #[post("/api/auth", data = "<password>")]
    pub fn login(password: String, cookies: &CookieJar<'_>) -> Status {
        let pwd = env::var("PASSWORD").expect("Please set the 'PASSWORD' env var.");

        if password != pwd {
            return Status::Unauthorized;
        }

        let token = make_token();

        cookies.add(Cookie::new("token", token));

        Status::Ok
    }

    pub fn validate(req: Request) -> bool {
        let token = match req.cookies().get("token") {
            Some(token) => token.value(),
            None => return false,
        };

        let dec = jwt::DecodingKey::from_base64_secret(
            &env::var("SECRET")
                .expect("Please set the 'SECRET' env var with a base64 secret."))
            .unwrap();
        
        

        true
    }

    fn make_token() -> String {
        let key = jwt::EncodingKey::from_base64_secret(
            &env::var("SECRET")
                .expect("Please set the 'SECRET' env var with a base64 secret."),
        )
        .unwrap();

        let time = Utc::now();

        let claims = Claims {
            iat: time.to_rfc3339(),
            exp: time
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap() // shouldn't fail
                .to_rfc3339(),
        };

        jwt::encode(&jwt::Header::default(), &claims, &key).unwrap()
    }
}

