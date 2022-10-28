use assert_cmd::Command;
use assert_fs::NamedTempFile;
use reqwest::{Client, StatusCode};
use rocket::tokio;

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

    let mut server = std::process::Command::new(env!("CARGO_BIN_EXE_pet-monitor-app"))
        .arg("--config")
        .arg(tmp.path())
        .arg("start")
        .arg("--port")
        .arg("8080")
        .spawn()
        .unwrap();

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| {
            server.kill().unwrap();
            e
        })
        .unwrap();

    let res = client
        .get("http://localhost:8080/api/config")
        .send()
        .await
        .map_err(|e| {
            server.kill().unwrap();
            e
        })
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .post("http://localhost:8080/api/login")
        .body("foo")
        .send()
        .await
        .map_err(|e| {
            server.kill().unwrap();
            e
        })
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .post("http://localhost:8080/api/login")
        .body("123")
        .send()
        .await
        .map_err(|e| {
            server.kill().unwrap();
            e
        })
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get("http://localhost:8080/api/config")
        .send()
        .await
        .map_err(|e| {
            server.kill().unwrap();
            e
        })
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    server.kill().unwrap();
}