//! This module provides structs for the password hash and JWT
//! signing secret.
//!
//! The structs are initialized in the `main` function and managed by Rocket
//! [`State`](rocket::State). This is why wrapper types are necessary.

use ring::rand::SecureRandom;
use std::{
    env, io,
    ops::{Deref, DerefMut},
};
use async_std::{fs, path::Path};

#[cfg(not(debug_assertions))]
pub const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret";
/// The path used to store the JWT signing secret.
///
/// It is `/var/local/lib/pet-monitor-app/jwt_secret` when compiled in release
/// mode, and `./jwt_secret` otherwise.
#[cfg(debug_assertions)]
pub const SECRET_PATH: &str = "./jwt_secret";

#[cfg(not(debug_assertions))]
pub const PASSWORD_PATH: &str = "/var/local/lib/pet-monitor-app/password";
/// The path used to store the JWT signing secret.
///
/// It is `/var/local/lib/pet-monitor-app/jwt_secret` when compiled in release
/// mode, and `./jwt_secret` otherwise.
#[cfg(debug_assertions)]
pub const PASSWORD_PATH: &str = "./password";

/// A wrapper struct for a password hash. A separate type is needed for Rocket state.
pub struct Password(pub String);

impl Password {
    /// Attempts to initialize the password hash.
    ///
    /// It first checks the `PASSWORD` env var. If it is set, it hashes it, writes
    /// the hash to [`PASSWORD_PATH`], and returns the hash. Otherwise, it reads the
    /// hash from [`PASSWORD_PATH`].
    ///
    /// # Example
    /// ```rust
    /// use std::env;
    /// use ring::rand::SystemRandom;
    /// use pet_monitor_app::secrets;
    /// # fn main() -> Result<(), impl std::error::Error> {
    ///
    /// // initialize RNG
    /// let rng = SystemRandom::new();
    ///
    /// let password = "123";
    /// env::set_var("PASSWORD", password);
    /// let hash = secrets::Password::new(&rng)?;
    ///
    /// let result = argon2::verify_encoded(&hash, password.as_bytes())?;
    /// assert!(result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function may panic if it does not have read or write access to
    /// `/var/local` and it was compiled for release.
    pub async fn new(rng: &impl SecureRandom) -> io::Result<Self> {
        if let Ok(p) = env::var("PASSWORD") {
            let config = argon2::Config {
                mem_cost: 8192, // KiB
                time_cost: 3,
                lanes: 4,
                variant: argon2::Variant::Argon2id,
                ..Default::default()
            };

            let mut buf = [0u8; 16];
            rng.fill(&mut buf)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

            let hash = argon2::hash_encoded(p.as_bytes(), &buf, &config).unwrap();

            if let Some(p) = Path::new(PASSWORD_PATH).parent() {
                fs::create_dir_all(p).await?;
            }

            fs::write(PASSWORD_PATH, &hash).await?;
            Ok(Self(hash))
        } else {
            fs::read_to_string(PASSWORD_PATH).await.map(|s| Self(s))
        }
    }
}

impl Deref for Password {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Password {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrapper struct for a password hash. A separate type is needed for Rocket state.
pub struct Secret(pub [u8; 32]);

impl Secret {
    /// A function to initialize the JWT signing secret.
    ///
    /// It first checks if the `REGEN_SECRET` env var is set to `true`. If it is,
    /// it generates a new random secret and writes that to [`SECRET_PATH`]. Otherwise,
    /// it attempts to read the secret from [`SECRET_PATH`].
    ///
    /// # Example
    /// ```rust
    /// use std::{env, fs};
    /// use ring::rand::SystemRandom;
    /// use pet_monitor_app::secrets;
    /// # fn main() -> Result<(), impl std::error::Error> {
    ///
    /// let rng = SystemRandom::new();
    ///
    /// let old_secret = secrets::Secret::new(&rng)?;
    /// env::set_var("REGEN_SECRET", "true");
    /// let secret = secrets::Secret::new(&rng)?;
    /// assert_ne!(*old_secret, *secret);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function may panic if it does not have read or write access to
    /// `/var/local` and it was compiled for release.
    pub async fn new(rng: &impl SecureRandom) -> io::Result<Self> {
        if env::var("REGEN_SECRET") == Ok("true".to_string()) {
            Self::new_secret(SECRET_PATH, rng).await
        } else {
            match fs::read(SECRET_PATH).await {
                Ok(s) => {
                    if let Ok(s) = s.try_into() {
                        Ok(Self(s))
                    } else {
                        Self::new_secret(SECRET_PATH, rng).await
                    }
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => Self::new_secret(SECRET_PATH, rng).await,
                    e => Err(io::Error::from(e)),
                },
            }
        }
    }

    /// Generates a secure random secret, writes it to `SECRET_PATH`, and returns it.
    async fn new_secret<P: AsRef<Path>>(path: P, rng: &impl SecureRandom) -> io::Result<Self> {
        if !path.as_ref().exists().await {
            if let Some(p) = path.as_ref().parent() {
                fs::create_dir_all(p).await?;
            }
        }
        let mut buf = [0u8; 32];
        rng.fill(&mut buf).unwrap();

        fs::write(path, buf).await?;
        Ok(Self(buf))
    }
}

impl Deref for Secret {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Secret {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
