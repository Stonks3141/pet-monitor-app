use ring::rand::{SecureRandom, SystemRandom};
use std::time::Instant;

use crate::secrets::ARGON2_CONFIG;

/// used for finding good argon2 params, make sure to add the `--release` flag
/// when running.
#[ignore]
#[test]
fn argon2_time() {
    let now = Instant::now();
    {
        let rand = SystemRandom::new();
        let mut salt = [0u8; 16];
        rand.fill(&mut salt).unwrap();
        argon2::hash_encoded("password".as_bytes(), &salt, &ARGON2_CONFIG).unwrap();
    }
    let elapsed = now.elapsed();
    println!("{:?}", elapsed);
}
