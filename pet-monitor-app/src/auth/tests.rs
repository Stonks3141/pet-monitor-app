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
