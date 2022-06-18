use super::*;
use jsonwebtoken as jwt;
use rocket::http::Status;
use std::env;

#[test]
fn authenticate_valid() {
    env::set_var("SECRET", "1234");
    env::set_var("PASSWORD", "123");

    let cookie = authenticate("123").unwrap();

    let res = validate_token(cookie.value());
    assert_eq!(res, Status::Ok, "status: {:?}", res);
}

#[test]
fn authenticate_invalid() {
    env::set_var("PASSWORD", "123");

    let res = authenticate("");
    assert_eq!(res, Err(Status::Unauthorized), "status: {:?}", res);
}

#[test]
#[should_panic]
/// `authenticate` should panic if the `PASSWORD` env var is not set
fn authenticate_panic() {
    env::remove_var("PASSWORD");
    _ = authenticate("");
}

#[test]
fn validate_valid_token() {
    let time = jwt::get_current_timestamp();

    let enc_key = jwt::EncodingKey::from_base64_secret("1234").unwrap();

    let valid_claims = Claims {
        iat: time,
        exp: time + 60 * 60 * 24,
    };

    let valid_token = jwt::encode(&jwt::Header::default(), &valid_claims, &enc_key).unwrap();

    let res = validate_token(&valid_token);
    assert_eq!(res, Status::Ok, "status: {:?}", res);
}

#[test]
fn validate_invalid_token() {
    env::set_var("SECRET", "1234");

    let time = jwt::get_current_timestamp();

    let enc_key = jwt::EncodingKey::from_base64_secret("1234").unwrap();

    let invalid_claims = Claims {
        iat: time - 2 * 60 * 60 * 24,
        exp: time - 60 * 60 * 24,
    };

    let invalid_token = jwt::encode(&jwt::Header::default(), &invalid_claims, &enc_key).unwrap();

    let res = validate_token(&invalid_token);
    assert_eq!(res, Status::Unauthorized, "status: {:?}", res);
}
