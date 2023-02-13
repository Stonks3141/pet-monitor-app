use super::{Assert, ReqBuilder, Request, ResponseAssert};
use pet_monitor_app::config::{Context, Tls};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Cmd<S> {
    subcmd: S,
    ctx: Context,
}

#[derive(Debug, Default)]
pub struct Start {
    no_stream: bool,
}

#[derive(Debug)]
pub struct StartRequest {
    request: Request,
    start: Start,
}

#[derive(Debug)]
pub struct SetPassword {
    password: String,
}

#[derive(Debug)]
pub struct RegenSecret;

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
            mem_cost: 16, // KiB
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
            .stderr(Stdio::piped())
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

    pub fn with_open_port(mut self) -> Self {
        let listener = TcpListener::bind(SocketAddr::new(self.ctx.host, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        self.ctx.port = port;
        if let Some(tls) = self.ctx.tls.as_mut() {
            let listener = TcpListener::bind(SocketAddr::new(self.ctx.host, 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            tls.port = port;
        }
        self
    }

    pub fn with_request(
        self,
        req_builder: impl FnOnce(ReqBuilder) -> Request,
    ) -> Cmd<StartRequest> {
        Cmd {
            subcmd: StartRequest {
                request: req_builder(ReqBuilder(self.ctx.clone())),
                start: self.subcmd,
            },
            ctx: self.ctx,
        }
    }

    pub fn assert(self) -> Assert<Context> {
        unimplemented!();
    }
}

impl Cmd<StartRequest> {
    pub fn assert(self) -> Assert<ResponseAssert> {
        let mut args = vec!["start"];
        if self.subcmd.start.no_stream {
            args.push("--no-stream");
        }

        let (mut child, conf_path) = self.run_command(&args);
        let start = Instant::now();
        while TcpStream::connect(SocketAddr::new(self.ctx.host, self.ctx.port)).is_err() {
            if start.elapsed() > Duration::from_secs(1) {
                panic!("Server failed to start in 1 second");
            }
        }
        let req = self.subcmd.request;
        let response = match req.body {
            Some(body) => req.request.send_string(&body),
            None => req.request.call(),
        };
        child.kill().unwrap();

        // display program output if tests fail
        let mut output = String::new();
        let mut stderr = child.stderr.unwrap();
        stderr.read_to_string(&mut output).unwrap();
        eprintln!("{output}");

        let response = response.unwrap();
        let ctx = toml::from_str(&std::fs::read_to_string(conf_path).unwrap()).unwrap();

        Assert(ResponseAssert { response, ctx })
    }
}

impl Cmd<SetPassword> {
    pub fn set_password(password: String) -> Self {
        Self {
            subcmd: SetPassword { password },
            ctx: Context::default(),
        }
    }

    pub fn assert(self) -> Assert<Context> {
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

    pub fn assert(self) -> Assert<Context> {
        unimplemented!();
    }
}
