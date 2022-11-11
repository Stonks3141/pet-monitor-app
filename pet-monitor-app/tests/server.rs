use assert_cmd::Command;
use assert_fs::NamedTempFile;
use reqwest::{Client, StatusCode};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn test_login() {
    let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();
    Command::cargo_bin("pet-monitor-app")
        .unwrap()
        .arg("--config")
        .arg(tmp.path())
        .arg("configure")
        .arg("--password")
        .arg("123")
        .assert()
        .success();

    let server = tokio::process::Command::new(env!("CARGO_BIN_EXE_pet-monitor-app"))
        .arg("--config")
        .arg(tmp.path())
        .arg("start")
        .arg("--port")
        .arg("8080")
        .arg("--no-stream")
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    // wait for the server to start up
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let client = Client::builder().cookie_store(true).build().unwrap();

    let res = client
        .get("http://127.0.0.1:8080/api/config")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .post("http://127.0.0.1:8080/api/login")
        .body("foo")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .post("http://127.0.0.1:8080/api/login")
        .body("123")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get("http://127.0.0.1:8080/api/config")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
