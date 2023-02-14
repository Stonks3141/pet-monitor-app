use pet_monitor_app::{auth, config::Context};
use std::time::Duration;

#[derive(Debug)]
pub struct ReqBuilder(pub(super) Context);

#[derive(Debug)]
pub struct Request {
    pub(super) request: ureq::Request,
    pub(super) body: Option<String>,
    pub(super) ctx: Context,
}

impl ReqBuilder {
    pub fn get(self, path: &str) -> Request {
        let agent = ureq::builder().redirects(0).build();
        Request {
            request: agent.get(&self.url(path)),
            body: None,
            ctx: self.0,
        }
    }

    pub fn post(self, path: &str) -> Request {
        let agent = ureq::builder().redirects(0).build();
        Request {
            request: agent.post(&self.url(path)),
            body: None,
            ctx: self.0,
        }
    }

    fn url(&self, path: &str) -> String {
        let scheme = if self.0.tls.is_some() {
            "https"
        } else {
            "http"
        };
        let host = self.0.host;
        let port = self.0.tls.as_ref().map_or(self.0.port, |it| it.port);

        format!("{scheme}://{host}:{port}{path}")
    }
}

impl Request {
    pub fn with_valid_token(mut self) -> Self {
        let token = auth::Token::new(Duration::from_secs(3600))
            .encode(&self.ctx.jwt_secret)
            .unwrap();
        self.request = self.request.set("Cookie", &format!("token={token}"));
        self
    }

    pub fn with_expired_token(mut self) -> Self {
        let token = auth::Token::new(Duration::ZERO)
            .encode(&self.ctx.jwt_secret)
            .unwrap();
        self.request = self.request.set("Cookie", &format!("token={token}"));
        self
    }

    pub fn form(mut self, form: &str) -> Self {
        self.body = Some(form.to_string());
        self.request = self
            .request
            .set("Content-Type", "application/x-www-form-urlencoded");
        self
    }
}
