use super::*;

#[test]
fn valid_token() {
    let secret = secrets::Secret([0u8; 32]);
    let token = Token::new();
    let token = token.to_string(&secret).unwrap();

    assert!(Token::from_str(&token, &secret).is_ok());
}

#[test]
fn invalid_token() {
    let secret = secrets::Secret([0u8; 32]);

    let utc = Utc::now();
    let claims = Claims {
        iat: (utc - Duration::days(2)).timestamp() as u64, // issued 2 days ago
        exp: (utc - Duration::days(1)).timestamp() as u64, // expired 1 day ago
    };

    let token = Token {
        header: jwt::Header::new(ALG),
        claims,
    };

    let token = token.to_string(&secret).unwrap();

    assert!(Token::from_str(&token, &secret).is_err());
}

#[test]
fn validate_correct_password() {
    let password = "password";
    let config = argon2::Config::default();
    let hash =
        secrets::Password(argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap());

    assert!(validate(password, &hash).unwrap());
}

#[test]
fn validate_incorrect_password() {
    let password = "password";
    let config = argon2::Config::default();
    let hash =
        secrets::Password(argon2::hash_encoded(password.as_bytes(), &[0u8; 16], &config).unwrap());

    assert!(!validate("paswurd", &hash).unwrap());
}
