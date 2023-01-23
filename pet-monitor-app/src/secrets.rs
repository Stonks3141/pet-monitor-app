use ring::rand::SecureRandom;
use tokio::task::spawn_blocking;

#[cfg(not(test))]
pub const ARGON2_CONFIG: argon2::Config = argon2::Config {
    ad: &[],
    hash_length: 32, // bytes
    lanes: 4,
    mem_cost: 8192, // KiB
    secret: &[],
    thread_mode: argon2::ThreadMode::Parallel,
    time_cost: 3,
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
};

#[cfg(test)]
pub const ARGON2_CONFIG: argon2::Config = argon2::Config {
    ad: &[],
    hash_length: 32, // bytes
    lanes: 1,
    mem_cost: 16, // KiB
    secret: &[],
    thread_mode: argon2::ThreadMode::Parallel,
    time_cost: 1,
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
};

/// Hashes a password with argon2 and a random 128-bit salt
pub async fn init_password(rng: &impl SecureRandom, password: &str) -> anyhow::Result<String> {
    let mut buf = [0u8; 16];
    // benched at 3.2 μs, don't need to `spawn_blocking`
    rng.fill(&mut buf)?;

    let password = password.to_string();

    spawn_blocking(move || {
        argon2::hash_encoded(password.as_bytes(), &buf, &ARGON2_CONFIG).map_err(Into::into)
    })
    .await?
}

/// Validates a password against a hash.
pub async fn validate(password: &str, hash: &str) -> anyhow::Result<bool> {
    let password = password.to_string();
    let hash = hash.to_string();
    spawn_blocking(move || argon2::verify_encoded(&hash, password.as_bytes()))
        .await?
        .map_err(Into::into)
}

/// Returns a randomly generated JWT secret
pub fn new_secret(rng: &impl SecureRandom) -> anyhow::Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    rng.fill(&mut buf)?;
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
}
