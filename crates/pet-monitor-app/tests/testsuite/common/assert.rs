use tower_cookies::Cookie;
use pet_monitor_app::{auth, config::Context};
use ureq::Response;

#[derive(Debug)]
pub struct Assert<T>(pub(super) T);

#[derive(Debug)]
pub struct ResponseAssert {
    pub(super) response: Response,
    pub(super) ctx: Context,
}

impl Assert<ResponseAssert> {
    pub fn context(mut self, f: impl FnOnce(Assert<Context>) -> Assert<Context>) -> Self {
        self.0.ctx = f(Assert(self.0.ctx)).0;
        self
    }

    pub fn ok(self) -> Self {
        assert_eq!(self.0.response.status(), 200);
        self
    }

    pub fn see_other(self, path: &str) -> Self {
        assert_eq!(self.0.response.status(), 303);
        assert_eq!(self.0.response.header("Location"), Some(path));
        self
    }

    pub fn permanent_redirect(self, path: &str) -> Self {
        assert_eq!(self.0.response.status(), 308);
        assert_eq!(self.0.response.header("Location"), Some(path));
        self
    }

    pub fn has_valid_token(self) -> Self {
        let cookie = self
            .0
            .response
            .header("Set-Cookie")
            .unwrap()
            .split("; ")
            .map(|it| Cookie::parse(it).unwrap())
            .find(|cookie| cookie.name() == "token")
            .unwrap();

        let token = auth::Token::decode(cookie.value(), &self.0.ctx.jwt_secret).unwrap();
        assert!(token.verify());

        self
    }
}
