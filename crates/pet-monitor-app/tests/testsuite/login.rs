use super::Cmd;

#[test]
fn login_correct_password() {
    Cmd::start()
        .no_stream()
        .with_open_port()
        .with_password("123")
        .with_request(|r| r.post("/login.html").form("password=123"))
        .assert()
        .see_other("/stream.html")
        .has_valid_token();
}

#[test]
fn login_incorrect_password() {
    Cmd::start()
        .no_stream()
        .with_open_port()
        .with_password("123")
        .with_request(|r| r.post("/login.html").form("password=foo"))
        .assert()
        .see_other("/login.html");
}
