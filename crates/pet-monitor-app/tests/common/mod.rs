#![allow(dead_code)]

use cookie::Cookie;
use pet_monitor_app::{
    auth,
    config::{Context, Tls},
};
use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
use ureq::Response;

#[derive(Debug)]
pub struct Cmd<S> {
    subcmd: S,
    ctx: Context,
}

#[derive(Debug, Default)]
pub struct Start {
    request: Option<Request>,
    no_stream: bool,
}

#[derive(Debug)]
pub struct SetPassword {
    password: String,
}

#[derive(Debug)]
pub struct RegenSecret;

#[derive(Debug)]
pub struct ReqBuilder(Context);

#[derive(Debug)]
pub struct Request {
    request: ureq::Request,
    ctx: Context,
}

#[derive(Debug)]
pub struct Assert<T>(T);

#[derive(Debug)]
pub struct StartAssert {
    response: Response,
    ctx: Context,
}

impl<S> Cmd<S> {
    pub fn with_config(mut self, ctx: Context) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn with_password(mut self, password: &str) -> Self {
        const ARGON2_CONFIG: argon2::Config = argon2::Config {
            ad: &[],
            hash_length: 32, // bytes
            lanes: 1,
            mem_cost: 4, // KiB
            secret: &[],
            thread_mode: argon2::ThreadMode::Parallel,
            time_cost: 1,
            variant: argon2::Variant::Argon2id,
            version: argon2::Version::Version13,
        };
        self.ctx.password_hash =
            argon2::hash_encoded(password.as_bytes(), &[0; 16], &ARGON2_CONFIG).unwrap();
        self
    }

    pub fn with_tls(mut self) -> Self {
        self.ctx.tls = Some(Tls {
            cert: PathBuf::from(format!("{}/cert.pem", env!("CARGO_MANIFEST_DIR"))),
            key: PathBuf::from(format!("{}/key.pem", env!("CARGO_MANIFEST_DIR"))),
            port: 8443,
        });
        self
    }

    fn run_command(&self, command: &[&str]) -> (Child, tempfile::TempPath) {
        let mut conf_file = tempfile::NamedTempFile::new().unwrap();
        conf_file
            .write_all(toml::to_string(&self.ctx).unwrap().as_bytes())
            .unwrap();
        let conf_path = conf_file.into_temp_path();

        let child = Command::new(env!("CARGO_BIN_EXE_pet-monitor-app"))
            .arg("--config")
            .arg(&conf_path)
            .args(command)
            .env("RUST_LOG", "debug")
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        (child, conf_path)
    }
}

impl Cmd<Start> {
    pub fn start() -> Self {
        Self {
            subcmd: Start::default(),
            ctx: Context::default(),
        }
    }

    pub fn no_stream(mut self) -> Self {
        self.subcmd.no_stream = true;
        self
    }

    pub fn with_request(mut self, req_builder: impl FnOnce(ReqBuilder) -> Request) -> Self {
        self.subcmd.request = Some(req_builder(ReqBuilder(self.ctx.clone())));
        self
    }

    pub fn run(self) -> Assert<StartAssert> {
        let mut args = vec!["start"];
        if self.subcmd.no_stream {
            args.push("--no-stream");
        }

        let (mut child, conf_path) = self.run_command(&args);
        std::thread::sleep(Duration::from_millis(100));
        let response = self.subcmd.request.map(|req| dbg!(req).request.call());
        child.kill().unwrap();

        let response = response.unwrap().unwrap();
        let ctx = toml::from_str(&std::fs::read_to_string(conf_path).unwrap()).unwrap();

        Assert(StartAssert { response, ctx })
    }
}

impl Cmd<SetPassword> {
    pub fn set_password(password: String) -> Self {
        Self {
            subcmd: SetPassword { password },
            ctx: Context::default(),
        }
    }

    pub fn run(self) -> Assert<SetPassword> {
        unimplemented!();
    }
}

impl Cmd<RegenSecret> {
    pub fn regen_secret() -> Self {
        Self {
            subcmd: RegenSecret,
            ctx: Context::default(),
        }
    }

    pub fn run(self) -> Assert<RegenSecret> {
        unimplemented!();
    }
}

impl ReqBuilder {
    pub fn get(self, path: &str) -> Request {
        let agent = ureq::builder().redirects(0).build();
        Request {
            request: agent.get(&self.url(path)),
            ctx: self.0,
        }
    }

    pub fn post(self, path: &str) -> Request {
        let agent = ureq::builder().redirects(0).build();
        Request {
            request: agent.get(&self.url(path)),
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

    pub fn form(self, form: &str) -> Self {
        unimplemented!();
    }
}

impl Assert<StartAssert> {
    pub fn response(mut self, f: impl FnOnce(Assert<Response>) -> Assert<Response>) -> Self {
        self.0.response = f(Assert(self.0.response)).0;
        self
    }
}

impl Assert<Response> {
    pub fn ok(self) -> Self {
        assert_eq!(self.0.status(), 200);
        self
    }

    pub fn see_other(self, path: &str) -> Self {
        assert_eq!(self.0.status(), 303);
        assert_eq!(self.0.header("Location"), Some(path));
        self
    }

    pub fn permanent_redirect(self, path: &str) -> Self {
        assert_eq!(self.0.status(), 308);
        assert_eq!(self.0.header("Location"), Some(path));
        self
    }

    pub fn has_valid_token(self) -> Self {
        let cookie = self
            .0
            .header("Set-Cookie")
            .unwrap()
            .split("; ")
            .map(|it| Cookie::parse(it).unwrap())
            .find(|cookie| cookie.name() == "token")
            .unwrap();

        // FIXME: use correct secret
        let token = auth::Token::decode(cookie.value(), &[0; 32]).unwrap();
        assert!(token.verify());

        self
    }
}
