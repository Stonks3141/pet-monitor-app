use super::Cmd;

#[test]
fn base_redirect_logged_out() {
    Cmd::start()
        .no_stream()
        .with_open_port()
        .with_request(|r| r.get("/"))
        .assert()
        .see_other("/login.html");
}

#[test]
fn base_redirect_logged_in() {
    Cmd::start()
        .no_stream()
        .with_open_port()
        .with_request(|r| r.get("/").with_valid_token())
        .assert()
        .see_other("/login.html");
}
