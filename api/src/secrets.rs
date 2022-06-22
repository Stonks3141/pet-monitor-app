use rand::prelude::*;
use once_cell::sync::Lazy;
use std::{io, fs, env, path::Path};

// This program expects to be run in a Docker container with access to /var
const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret";
const PASSWORD_PATH: &str = "/var/local/lib/pet-monitor-app/password";

pub static PASSWORD_HASH: Lazy<String> = Lazy::new(|| {
    if cfg!(test) {
        let config = argon2::Config {
            mem_cost: 16384,
            ..Default::default()
        };

        argon2::hash_encoded(b"hi", &random::<[u8; 16]>(), &config).unwrap()
    } else if let Ok(p) = env::var("PASSWORD") {
        fs::write(PASSWORD_PATH, &p).unwrap();
        p
    } else {
        fs::read_to_string(PASSWORD_PATH).expect("Failed to read password hash.")
    }
});

pub static SECRET: Lazy<[u8; 32]> = Lazy::new(|| {
    get_secret()
        .expect("Failed to initialize JWT secret. Is the program running in a Docker container?")
});

fn get_secret() -> io::Result<[u8; 32]> {
    match fs::read(SECRET_PATH) {
        Ok(s) => {
            if let Ok(s) = s.try_into() {
                Ok(s)
            } else {
                init_secret(SECRET_PATH)
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                init_secret(SECRET_PATH)
            },
            e => Err(io::Error::from(e)),
        },
    }
}

fn init_secret<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    if !path.as_ref().exists() {
        if let Some(p) = path.as_ref().parent() {
            fs::create_dir_all(p)?;
        }
    }

    let rand = random::<[u8; 32]>(); // 256-bit secret
    fs::write(path, rand)?;
    Ok(rand)
}
