use assert_cmd::Command;
use assert_fs::NamedTempFile;
use reqwest::{Client, StatusCode};
use std::process::Stdio;

#[tokio::test]
async fn test_login() {
    let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();
    Command::cargo_bin("pet-monitor-app")
        .unwrap()
        .arg("--config")
        .arg(tmp.path())
        .arg("set-password")
        .arg("123")
        .assert()
        .success();

    let _server = tokio::process::Command::new(env!("CARGO_BIN_EXE_pet-monitor-app"))
        .arg("--config")
        .arg(tmp.path())
        .arg("start")
        .arg("--port")
        .arg("8080")
        .arg("--no-stream")
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let client = Client::builder().cookie_store(true).build().unwrap();

    // wait for the server to start up
    let mut i = 0;
    while client.get("http://127.0.0.1:8080").send().await.is_err() {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        i += 1;
        if i > 100 {
            return;
        }
    }

    let res = client
        .get("http://127.0.0.1:8080/api/config")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

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
