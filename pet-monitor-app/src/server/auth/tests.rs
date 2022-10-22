use super::*;
use crate::config::Context;
use crate::server::provider::Provider;
use quickcheck::{quickcheck, Arbitrary, Gen};
use rocket::http::{Cookie, Header, Method, Status};
use rocket::local::blocking::Client;
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

macro_rules! make_routes {
    (@internal $method:ident, $name:ident) => {
        #[$method("/")]
        fn $name(token: Token) -> Status {
            if token.verify() {
                Status::Ok
            } else {
                Status::Unauthorized
            }
        }
    };
    ( $( $method:ident, $name:ident ),* $(,)? ) => {
        $( make_routes!(@internal $method, $name); )*
    }
}

make_routes! {
    get, get_route,
    post, post_route,
    put, put_route,
    delete, delete_route,
    patch, patch_route,
    head, head_route,
    options, options_route,
}

#[derive(Debug, Clone)]
struct ArbMethod(Method);

impl Arbitrary for ArbMethod {
    fn arbitrary(g: &mut Gen) -> Self {
        match u32::arbitrary(g) % 7 {
            0 => Self(Method::Get),
            1 => Self(Method::Put),
            2 => Self(Method::Post),
            3 => Self(Method::Delete),
            4 => Self(Method::Patch),
            5 => Self(Method::Head),
            6 => Self(Method::Options),
            _ => unreachable!(),
        }
    }
}

fn client() -> Client {
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
                options_route,
            ],
        )
        .manage(Provider::new(Context::default()));
    Client::tracked(rocket).unwrap()
}

quickcheck! {
    fn qc_guard_no_csrf(method: ArbMethod, is_valid: bool) -> bool {
        let client = client();

        let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
        let res = client
            .req(method.0, "/")
            .cookie(Cookie::new("token", &token))
            .dispatch();

        let expected = match method.0 {
            Method::Get | Method::Head | Method::Options | Method::Trace if is_valid => Status::Ok,
            _ => Status::Unauthorized,
        };
        res.status() == expected
    }

    fn qc_guard_invalid_csrf(method: ArbMethod, is_valid: bool) -> bool {
        let client = client();

        let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
        let res = client
            .req(method.0, "/")
            .cookie(Cookie::new("token", &token))
            .header(Header::new("x-csrf-token", "foo"))
            .dispatch();

        let expected = match method.0 {
            Method::Get | Method::Head | Method::Options | Method::Trace if is_valid => Status::Ok,
            _ => Status::Unauthorized,
        };
        res.status() == expected
    }

    fn qc_guard_valid_csrf(method: ArbMethod, is_valid: bool) -> bool {
        let client = client();

        let token = make_token(is_valid).to_string(&[0; 32]).unwrap();
        let res = client
            .req(method.0, "/")
            .cookie(Cookie::new("token", &token))
            .header(Header::new("x-csrf-token", token))
            .dispatch();

        let expected = if is_valid {
            Status::Ok
        } else {
            Status::Unauthorized
        };
        res.status() == expected
    }

    fn qc_token_validity(is_valid: bool) -> bool {
        let token = make_token(is_valid);
        token.verify() == is_valid
    }

    fn qc_token_parse_validity(is_valid: bool) -> bool {
        let secret = [0u8; 32];
        let token = make_token(is_valid);
        let token = token.to_string(&secret).unwrap();

        match Token::from_str(&token, &secret) {
            Ok(token) => token.verify() == is_valid,
            Err(_) => !is_valid,
        }
    }
}
