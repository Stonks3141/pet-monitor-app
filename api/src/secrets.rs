use once_cell::sync::OnceCell;
use ring::rand::{SecureRandom, SystemRandom};
use std::{env, fs, io, path::Path};

// This program expects to be run in a Docker container with access to /var/local
const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret";
const PASSWORD_PATH: &str = "/var/local/lib/pet-monitor-app/password";

pub static RAND: OnceCell<SystemRandom> = OnceCell::new();
pub static PASSWORD_HASH: OnceCell<String> = OnceCell::new();
pub static SECRET: OnceCell<[u8; 32]> = OnceCell::new();

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
        fs::create_dir_all("/var/local/lib/pet-monitor-app")?;
        fs::write(PASSWORD_PATH, &hash)?;
        Ok(p)
    } else {
        fs::read_to_string(PASSWORD_PATH)
    }
}

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
