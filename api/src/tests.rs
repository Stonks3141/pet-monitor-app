use ring::rand::{SecureRandom, SystemRandom};
use std::time::Instant;

/// used for finding good argon2 params
#[test]
fn argon2_time() {
    let now = Instant::now();
    {
        let rand = SystemRandom::new();
        let mut buf = [0u8; 16];
        rand.fill(&mut buf).unwrap();

        let config = argon2::Config {
            mem_cost: 8192, // KiB
            time_cost: 2,
            lanes: 4,
            variant: argon2::Variant::Argon2id,
            ..Default::default()
        };

        argon2::hash_encoded("password".as_bytes(), &buf, &config).unwrap();
    }
    let elapsed = now.elapsed();
    println!("{:?}", elapsed);
}
