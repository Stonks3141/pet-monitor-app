use super::*;
use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};
use rocket::tokio;

macro_rules! is_valid {
    (@path $val:expr) => {{
        !($val).clone().map_or(false, |x| {
            x.to_string_lossy().starts_with("-") || x.as_os_str().is_empty()
        })
    }};
    (@string $val:expr) => {{
        !($val)
            .clone()
            .map_or(false, |x| x.starts_with("-") || x.is_empty())
    }};
}

impl Cmd {
    fn is_valid(&self) -> bool {
        is_valid!(@path self.conf_path) && self.command.is_valid()
    }
}

impl SubCmd {
    fn is_valid(&self) -> bool {
        match self.clone() {
            Self::Configure { password, .. } => is_valid!(@string password),
            Self::Start { cert, key, .. } => is_valid!(@path cert) && is_valid!(@path key),
        }
    }
}

fn level_from_int(x: u32) -> Level {
    match x % 5 {
        0 => Level::Trace,
        1 => Level::Debug,
        2 => Level::Info,
        3 => Level::Warn,
        4 => Level::Error,
        _ => panic!(),
    }
}

fn level_to_int(x: Level) -> u32 {
    match x {
        Level::Trace => 0,
        Level::Debug => 1,
        Level::Info => 2,
        Level::Warn => 3,
        Level::Error => 4,
    }
}

macro_rules! shrink_tuple {
    ($struct:ident { $( $field:ident ),* $(,)? }) => {
        Box::new(( $( $field, )* ).shrink().map(|( $( $field, )* )| $struct { $( $field, )* }))
    };
}

impl Arbitrary for Cmd {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            command: Arbitrary::arbitrary(g),
            conf_path: Arbitrary::arbitrary(g),
            log_level: level_from_int(Arbitrary::arbitrary(g)),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.command.clone(),
                self.conf_path.clone(),
                level_to_int(self.log_level),
            )
                .shrink()
                .map(|(command, conf_path, log_level)| Self {
                    command,
                    conf_path,
                    log_level: level_from_int(log_level),
                }),
        )
    }
}

impl Arbitrary for SubCmd {
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Configure {
                password: Arbitrary::arbitrary(g),
                regen_secret: Arbitrary::arbitrary(g),
            }
        } else {
            Self::Start {
                tls: Arbitrary::arbitrary(g),
                tls_port: Arbitrary::arbitrary(g),
                cert: Arbitrary::arbitrary(g),
                key: Arbitrary::arbitrary(g),
                port: Arbitrary::arbitrary(g),
                stream: Arbitrary::arbitrary(g),
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use SubCmd::{Configure, Start};
        match self.clone() {
            Self::Configure {
                password,
                regen_secret,
            } => shrink_tuple!(Configure {
                password,
                regen_secret
            }),
            Self::Start {
                tls,
                tls_port,
                cert,
                key,
                port,
                stream,
            } => shrink_tuple!(Start {
                tls,
                tls_port,
                cert,
                key,
                port,
                stream,
            }),
        }
    }
}

fn make_args(cmd: &Cmd) -> Vec<String> {
    let mut args = vec!["pet-monitor-app".to_string()];

    if let Some(conf_path) = &cmd.conf_path {
        args.push("--config".to_string());
        args.push(conf_path.clone().into_os_string().into_string().unwrap());
    }

    match cmd.log_level {
        Level::Trace => args.push("-vv".to_string()),
        Level::Debug => args.push("-v".to_string()),
        Level::Info => (),
        Level::Warn => args.push("-q".to_string()),
        Level::Error => args.push("-qq".to_string()),
    }

    match &cmd.command {
        SubCmd::Configure {
            password,
            regen_secret,
        } => {
            args.push("configure".to_string());
            if *regen_secret {
                args.push("--regen-secret".to_string());
            }
            if let Some(password) = password {
                args.push("--password".to_string());
                args.push(password.clone());
            }
        }
        SubCmd::Start {
            tls,
            port,
            tls_port,
            cert,
            key,
            stream,
        } => {
            args.push("start".to_string());
            if let Some(tls) = tls {
                args.push("--tls".to_string());
                args.push(tls.to_string());
            }
            if let Some(port) = port {
                args.push("--port".to_string());
                args.push(port.to_string());
            }
            if let Some(tls_port) = tls_port {
                args.push("--tls-port".to_string());
                args.push(tls_port.to_string());
            }
            if let Some(cert) = cert {
                args.push("--cert".to_string());
                args.push(cert.to_owned().into_os_string().into_string().unwrap());
            }
            if let Some(key) = key {
                args.push("--key".to_string());
                args.push(key.to_owned().into_os_string().into_string().unwrap());
            }
            if !stream {
                args.push("--no-stream".to_string());
            }
        }
    }

    args
}

#[test]
fn verify_cmd() {
    cmd().debug_assert();
}

quickcheck! {
    fn qc_cmd(cmd: Cmd) -> TestResult {
        if !cmd.is_valid() {
            TestResult::discard()
        } else {
            let args = make_args(&cmd);
            TestResult::from_bool(cmd == parse_args(args))
        }
    }
}

#[tokio::test]
async fn merge_tls_cfg() -> anyhow::Result<()> {
    let cmd = Cmd {
        command: SubCmd::Start {
            tls: Some(true),
            tls_port: Some(8443),
            cert: Some(PathBuf::from("cert.pem")),
            key: Some(PathBuf::from("key.key")),
            port: None,
            stream: false,
        },
        conf_path: None,
        log_level: Level::Info,
    };

    let ctx = Context::default();
    let ctx = merge_ctx(&cmd, ctx).await?;

    let expected = Context {
        tls: Some(Tls {
            port: 8443,
            cert: PathBuf::from("cert.pem"),
            key: PathBuf::from("key.key"),
        }),
        ..Default::default()
    };

    assert_eq!(ctx, expected);

    Ok(())
}

#[tokio::test]
async fn merge_no_tls() -> anyhow::Result<()> {
    let cmd = Cmd {
        command: SubCmd::Start {
            tls: Some(false),
            tls_port: Some(8443),
            cert: None,
            key: None,
            port: None,
            stream: false,
        },
        conf_path: None,
        log_level: Level::Info,
    };

    let ctx = Context {
        tls: Some(Tls {
            port: 8443,
            cert: PathBuf::from("cert.pem"),
            key: PathBuf::from("key.key"),
        }),
        ..Default::default()
    };
    let ctx = merge_ctx(&cmd, ctx).await?;

    let expected = Context::default();

    assert_eq!(ctx, expected);

    Ok(())
}

#[tokio::test]
async fn merge_invalid_tls() {
    let cmd = Cmd {
        command: SubCmd::Start {
            tls: Some(true),
            tls_port: Some(8443),
            cert: Some(PathBuf::from("cert.pem")),
            key: None,
            port: None,
            stream: false,
        },
        conf_path: None,
        log_level: Level::Info,
    };

    let ctx = Context::default();
    let res = merge_ctx(&cmd, ctx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn merge_jwt_regen() -> anyhow::Result<()> {
    let cmd = Cmd {
        command: SubCmd::Configure {
            password: None,
            regen_secret: true,
        },
        conf_path: None,
        log_level: Level::Info,
    };

    let ctx = Context::default();
    let new_ctx = merge_ctx(&cmd, ctx.clone()).await?;

    assert_ne!(ctx.jwt_secret, new_ctx.jwt_secret);

    Ok(())
}

#[tokio::test]
async fn merge_new_password() -> anyhow::Result<()> {
    let password = "foo";

    let cmd = Cmd {
        command: SubCmd::Configure {
            password: Some(password.to_string()),
            regen_secret: false,
        },
        conf_path: None,
        log_level: Level::Info,
    };

    let ctx = Context::default();
    let ctx = merge_ctx(&cmd, ctx.clone()).await?;

    assert!(crate::secrets::validate(password, &ctx.password_hash)
        .await
        .unwrap());

    Ok(())
}
