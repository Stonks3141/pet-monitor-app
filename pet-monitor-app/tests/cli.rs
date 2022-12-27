use assert_cmd::Command;
use assert_fs::{prelude::*, NamedTempFile};
use predicates::prelude::*;

#[test]
fn set_password() {
    let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();
    Command::cargo_bin("pet-monitor-app")
        .unwrap()
        .arg("--config")
        .arg(tmp.path())
        .arg("set-password")
        .arg("123")
        .assert()
        .success();
    tmp.assert(predicate::path::exists());
    tmp.assert(predicate::path::is_file());
}

#[test]
fn regen_secret() {
    let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();
    Command::cargo_bin("pet-monitor-app")
        .unwrap()
        .arg("--config")
        .arg(tmp.path())
        .arg("regen-secret")
        .assert()
        .success();
    tmp.assert(predicate::path::exists());
    tmp.assert(predicate::path::is_file());
}
