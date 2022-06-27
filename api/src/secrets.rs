// Pet Montitor App
// Copyright (C) 2022  Samuel Nystrom
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This module provides statics for a CSPRNG and the password hash and JWT
//! signing secret, as well as initialization functions.
//!
//! The statics are initialized with [`once_cell::sync::OnceCell`] at the
//! beginning of the binary crate's `main` function using [`init_secret()`] and
//! [`init_pwd()`].

use once_cell::sync::OnceCell;
use ring::rand::{SecureRandom, SystemRandom};
use std::{env, fs, io, path::Path};

/// The path used to store the JWT signing secret.
///
/// This program expects to be run in a Docker container with access to
/// `/var/local` and panics if it cannot read or write files there.
#[cfg(not(debug_assertions))]
pub const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret";
#[cfg(debug_assertions)]
pub const SECRET_PATH: &str = "./jwt_secret";

/// The path used to store the password hash.
///
/// This program expects to be run in a Docker container with access to
/// `/var/local` and panics if it cannot read or write files there.
#[cfg(not(debug_assertions))]
pub const PASSWORD_PATH: &str = "/var/local/lib/pet-monitor-app/password";
#[cfg(debug_assertions)]
pub const PASSWORD_PATH: &str = "./password";

/// A CSPRNG ([`ring::rand::SystemRandom`]).
///
///
/// Access it with `RAND.get().unwrap()`. The `.unwrap()` is safe
/// because the static is initialized at the beginning of `main()`.
pub static RAND: OnceCell<SystemRandom> = OnceCell::new();

/// The argon2-hashed password.
///
/// Loaded from `/var/local/lib/pet-monitor-app/password`.
/// If the `PASSWORD` env var is set, then the old password will be overwritten
/// with the new one.
///
/// Access it with `PASSWORD_HASH.get().unwrap()`. The `.unwrap()` is safe
/// because the static is initialized at the beginning of `main()`.
pub static PASSWORD_HASH: OnceCell<String> = OnceCell::new();

/// The secret value used for signing JWTs.
///
/// Loaded from `/var/local/lib/pet-monitor-app/jwt_secret`.
/// If the `REGEN_SECRET` env var is set to `true`, a new secret will be
/// generated and stored.
///
/// Access it with `SECRET.get().unwrap()`. The `.unwrap()` is safe
/// because the static is initialized at the beginning of `main()`.
pub static SECRET: OnceCell<[u8; 32]> = OnceCell::new();

/// Attempts to initialize the password hash.
///
/// It first checks the `PASSWORD` env var. If it is set, it hashes it, writes
/// the hash to [`PASSWORD_PATH`], and returns the hash. Otherwise, it reads the
/// hash from [`PASSWORD_PATH`].
///
/// # Examples
/// Basic usage:
/// ```
/// use std::env;
/// use ring::rand::SystemRandom;
/// use pet_monitor_app::secrets;
///
/// // initialize RNG
/// secrets::RAND.set(SystemRandom::new()).unwrap_or(());
///
/// let password = "123";
/// env::set_var("PASSWORD", password);
/// let hash = secrets::init_pwd().unwrap();
///
/// let result = argon2::verify_encoded(&hash, password.as_bytes()).unwrap();
/// assert!(result);
/// ```
///
/// # Panics
/// This function may panic if it does not have read or write access to
/// `/var/local` and it was compiled for release.
pub fn init_pwd() -> io::Result<String> {
    if let Ok(p) = env::var("PASSWORD") {
        let config = argon2::Config {
            mem_cost: 8192, // KiB
            time_cost: 3,
            lanes: 4,
            variant: argon2::Variant::Argon2id,
            ..Default::default()
        };
        let mut buf = [0u8; 16];
        RAND.get().unwrap().fill(&mut buf).unwrap();
        let hash = argon2::hash_encoded(p.as_bytes(), &buf, &config).unwrap();

        if let Some(p) = Path::new(PASSWORD_PATH).parent() {
            fs::create_dir_all(p)?;
        }

        fs::write(PASSWORD_PATH, &hash)?;
        Ok(hash)
    } else {
        fs::read_to_string(PASSWORD_PATH)
    }
}

/// A function to initialize the JWT signing secret.
///
/// It first checks if the `REGEN_SECRET` env var is set to `true`. If it is,
/// it generates a new random secret and writes that to [`SECRET_PATH`]. Otherwise,
/// it attempts to read the secret from [`SECRET_PATH`].
///
/// # Examples
/// Basic usage:
/// ```
/// use std::{env, fs};
/// use ring::rand::SystemRandom;
/// use pet_monitor_app::secrets;
///
/// // initialize RNG
/// secrets::RAND.set(SystemRandom::new()).unwrap_or(());
///
/// let old_secret = secrets::init_secret().unwrap();
/// env::set_var("REGEN_SECRET", "true");
/// let secret = secrets::init_secret().unwrap();
/// assert_ne!(old_secret, secret);
/// ```
///
/// # Panics
/// This function may panic if it does not have read or write access to
/// `/var/local` and it was compiled for release.
pub fn init_secret() -> io::Result<[u8; 32]> {
    if env::var("REGEN_SECRET") == Ok("true".to_string()) {
        new_secret(SECRET_PATH)
    } else {
        match fs::read(SECRET_PATH) {
            Ok(s) => {
                if let Ok(s) = s.try_into() {
                    Ok(s)
                } else {
                    new_secret(SECRET_PATH)
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => new_secret(SECRET_PATH),
                e => Err(io::Error::from(e)),
            },
        }
    }
}

/// Generates a secure random secret, writes it to `SECRET_PATH`, and returns it.
fn new_secret<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    if !path.as_ref().exists() {
        if let Some(p) = path.as_ref().parent() {
            fs::create_dir_all(p)?;
        }
    }
    let mut buf = [0u8; 32];
    RAND.get().unwrap().fill(&mut buf).unwrap();

    fs::write(path, buf)?;
    Ok(buf)
}
