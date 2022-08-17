//! This module provides structs for the password hash and JWT
//! signing secret.
//!
//! The structs are initialized in the `main` function and managed by Rocket
//! [`State`](rocket::State). This is why wrapper types are necessary.

use quick_error::quick_error;
use ring::rand::SecureRandom;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Rng {
            display("PRNG error (unspecified)")
            from(ring::error::Unspecified)
        }
        Hash(err: argon2::Error) {
            source(err)
            display("Hashing error: {}", err)
            from()
        }
        Other {
            display("Other error")
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn init_password(rng: &impl SecureRandom, password: &str) -> Result<String> {
    let mut buf = [0u8; 16];
    rng.fill(&mut buf).map_err(Into::<Error>::into)?;
    let config = argon2::Config {
        mem_cost: 8192, // KiB
        time_cost: 3,
        lanes: 4,
        variant: argon2::Variant::Argon2id,
        ..Default::default()
    };

    argon2::hash_encoded(password.as_bytes(), &buf, &config).map_err(Into::into)
}

pub fn new_secret(rng: &impl SecureRandom) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    rng.fill(&mut buf).map_err(Into::<Error>::into)?;
    Ok(buf)
}