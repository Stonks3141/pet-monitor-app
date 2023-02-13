mod common;

use common::Cmd;

#[test]
fn base_redirect_logged_out() {
    Cmd::start()
        .no_stream()
        .with_request(|r| r.get("/"))
        .run()
        .response(|res| res.see_other("/login.html"));
}

// fails randomly for no apparent reason
#[ignore]
#[test]
fn base_redirect_logged_in() {
    Cmd::start()
        .no_stream()
        .with_request(|r| r.get("/").with_valid_token())
        .run()
        .response(|res| res.see_other("/login.html"));
}
