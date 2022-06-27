use super::*;

#[test]
fn valid_token() {
    secrets::SECRET.set([0; 32]).unwrap_or(());

    let token = Token::new();
    let token = String::try_from(token).unwrap();

    assert!(Token::from_str(&token).is_ok());
}

#[test]
fn invalid_token() {
    secrets::SECRET.set([0; 32]).unwrap_or(());

    let utc = Utc::now();
    let claims = Claims {
        iat: (utc - Duration::days(2)).timestamp() as u64, // issued 2 days ago
        exp: (utc - Duration::days(1)).timestamp() as u64, // expired 1 day ago
    };

    let token = Token {
        header: jwt::Header::new(ALG),
        claims,
    };

    let token = String::try_from(token).unwrap();

    assert!(Token::from_str(&token).is_err());
}

#[test]
fn validate_correct_password() {
    let config = argon2::Config::default();
    let hash = argon2::hash_encoded("password".as_bytes(), &[0u8; 16], &config).unwrap();
    secrets::PASSWORD_HASH.set(hash).unwrap_or(());

    assert!(validate("password").unwrap());
}

#[test]
fn validate_incorrect_password() {
    let config = argon2::Config::default();
    let hash = argon2::hash_encoded("password".as_bytes(), &[0u8; 16], &config).unwrap();
    secrets::PASSWORD_HASH.set(hash).unwrap_or(());

    assert!(!validate("paswurd").unwrap());
}
