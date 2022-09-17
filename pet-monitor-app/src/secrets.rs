//! This module provides structs for the password hash and JWT
//! signing secret.
//!
//! The structs are initialized in the `main` function and managed by Rocket
//! [`State`](rocket::State). This is why wrapper types are necessary.

use quick_error::quick_error;
use ring::rand::SecureRandom;
use rocket::tokio::task::spawn_blocking;

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
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn init_password(rng: &impl SecureRandom, password: &str) -> Result<String> {
    let mut buf = [0u8; 16];
     // benched at 3.2 μs, don't need to `spawn_blocking`
    rng.fill(&mut buf).map_err(Into::<Error>::into)?;
    let config = argon2::Config {
        mem_cost: 8192, // KiB
        time_cost: 3,
        lanes: 4,
        variant: argon2::Variant::Argon2id,
        ..Default::default()
    };

    let password = password.to_string();

    spawn_blocking(move || {
        argon2::hash_encoded(password.as_bytes(), &buf, &config).map_err(|e| e.into())
    }).await.unwrap()
}

pub fn new_secret(rng: &impl SecureRandom) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    rng.fill(&mut buf).map_err(Into::<Error>::into)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use rocket::tokio;

    #[tokio::test]
    async fn test_hash() {
        let password = "password";
        let rng = SystemRandom::new();
        let hash = init_password(&rng, &password).await.unwrap();
        assert!(argon2::verify_encoded(&hash, password.as_bytes()).unwrap());
    }

    #[tokio::test]
    async fn test_hash_invalid() {
        let password = "password";
        let rng = SystemRandom::new();
        let hash = init_password(&rng, &password).await.unwrap();
        assert!(!argon2::verify_encoded(&hash, "paswurd".as_bytes()).unwrap());
    }
}
