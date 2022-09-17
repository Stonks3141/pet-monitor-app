use super::*;
use crate::config::Context;
use crate::server::provider::Provider;
use proptest::prelude::*;
use rocket::http::{Cookie, Header, Method, Status};
use rocket::local::asynchronous::Client;
use rocket::tokio;
use rocket::{delete, get, head, options, patch, post, put};

fn make_token(is_valid: bool) -> Token {
    if is_valid {
        Token::new(Duration::days(1))
    } else {
        let utc = Utc::now();
        let claims = Claims {
            iat: (utc - Duration::days(2)).timestamp() as u64, // issued 2 days ago
            exp: (utc - Duration::days(1)).timestamp() as u64, // expired 1 day ago
        };

        Token {
            header: jwt::Header::new(ALG),
            claims,
        }
    }
}

#[get("/")]
fn get_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[post("/")]
fn post_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[put("/")]
fn put_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[delete("/")]
fn delete_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[patch("/")]
fn patch_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[head("/")]
fn head_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[options("/")]
fn options_route(token: Token) -> Status {
    if token.verify() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

fn method_strategy() -> impl Strategy<Value = Method> {
    prop_oneof![
        Just(Method::Get),
        Just(Method::Put),
        Just(Method::Post),
        Just(Method::Delete),
        Just(Method::Patch),
        Just(Method::Head),
        Just(Method::Options),
        // Just(Method::Trace),
        // Just(Method::Connect),
    ]
}

async fn client() -> Client {
    let rocket = rocket::build()
        .mount(
            "/",
            rocket::routes![
                get_route,
                post_route,
                put_route,
                delete_route,
                patch_route,
                head_route,
                options_route
            ],
        )
        .manage(Provider::new(Context::default(), |_| {}));
    Client::tracked(rocket).await.unwrap()
}

async fn req_guard_no_csrf(client: &Client, method: &Method, is_valid: bool) -> bool {
    let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
    let res = client
        .req(*method, "/")
        .cookie(Cookie::new("token", &token))
        .dispatch()
        .await;
    let expected = match *method {
        Method::Get | Method::Head | Method::Options | Method::Trace if is_valid => Status::Ok,
        _ => Status::Unauthorized,
    };
    res.status() == expected
}

async fn req_guard_invalid_csrf(client: &Client, method: &Method, is_valid: bool) -> bool {
    let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
    let res = client
        .req(*method, "/")
        .cookie(Cookie::new("token", &token))
        .header(Header::new("x-csrf-token", "foo"))
        .dispatch()
        .await;
    let expected = match *method {
        Method::Get | Method::Head | Method::Options | Method::Trace if is_valid => Status::Ok,
        _ => Status::Unauthorized,
    };
    res.status() == expected
}

async fn req_guard_valid_csrf(client: &Client, method: &Method, is_valid: bool) -> bool {
    let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
    let res = client
        .req(*method, "/")
        .cookie(Cookie::new("token", &token))
        .header(Header::new("x-csrf-token", token))
        .dispatch()
        .await;
    let expected = if is_valid {
        Status::Ok
    } else {
        Status::Unauthorized
    };
    res.status() == expected
}

#[test]
fn valid_token() {
    let secret = [0u8; 32];
    let token = make_token(true);
    let token = token.to_string(&secret).unwrap();

    assert!(Token::from_str(&token, &secret).is_ok());
}

#[test]
fn invalid_token() {
    let secret = [0u8; 32];
    let token = make_token(false);

    let token = token.to_string(&secret).unwrap();

    assert!(Token::from_str(&token, &secret).is_err());
}

#[tokio::test]
async fn valid_token_get() {
    let client = client().await;
    assert!(req_guard_no_csrf(&client, &Method::Get, true).await);
}

#[tokio::test]
async fn invalid_token_get() {
    let client = client().await;
    assert!(req_guard_no_csrf(&client, &Method::Get, false).await);
}

#[tokio::test]
async fn invalid_token_get_csrf() {
    let client = client().await;
    assert!(req_guard_valid_csrf(&client, &Method::Get, false).await);
}

#[tokio::test]
async fn valid_token_post_no_csrf() {
    let client = client().await;
    assert!(req_guard_no_csrf(&client, &Method::Post, true).await);
}

#[tokio::test]
async fn valid_token_post_csrf() {
    let client = client().await;
    assert!(req_guard_valid_csrf(&client, &Method::Post, true).await);
}

#[tokio::test]
async fn valid_token_post_invalid_csrf() {
    let client = client().await;
    assert!(req_guard_invalid_csrf(&client, &Method::Post, true).await);
}

proptest! {
    #[test]
    fn prop_req_guard_invalid_csrf(method in method_strategy(), is_valid: bool) {
        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            assert!(req_guard_invalid_csrf(&client().await, &method, is_valid).await);
        });
    }
}

proptest! {
    #[test]
    fn prop_req_guard_no_csrf(method in method_strategy(), is_valid: bool) {
        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            assert!(req_guard_no_csrf(&client().await, &method, is_valid).await);
        });
    }
}

proptest! {
    #[test]
    fn prop_req_guard_valid_csrf(method in method_strategy(), is_valid: bool) {
        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            assert!(req_guard_valid_csrf(&client().await, &method, is_valid).await);
        });
    }
}

#[tokio::test]
async fn validate_correct_password() {
    let password = "password";
    let config = argon2::Config {
        mem_cost: 128, // KiB
        time_cost: 1,
        lanes: 1,
        variant: argon2::Variant::Argon2id,
        ..Default::default()
    };
    let hash = argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap();

    assert!(validate(password, &hash).await.unwrap());
}

#[tokio::test]
async fn validate_incorrect_password() {
    let password = "password";
    let config = argon2::Config {
        mem_cost: 128, // KiB
        time_cost: 1,
        lanes: 1,
        variant: argon2::Variant::Argon2id,
        ..Default::default()
    };
    let hash = argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap();

    assert!(!validate("paswurd", &hash).await.unwrap());
}
