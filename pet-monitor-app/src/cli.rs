//! This module handles command-line interactions with the application.

use crate::config::{Context, Tls};
use crate::secrets;
use clap::builder::{ArgAction, Command, ValueHint};
use clap::{arg, value_parser};
use log::Level;
use ring::rand::SystemRandom;
use std::path::PathBuf;

/// A struct for command-line args
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    pub command: SubCmd,
    pub conf_path: Option<PathBuf>,
    pub log_level: Level,
}

/// The CLI subcommand
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubCmd {
    Start {
        tls: Option<bool>,
        tls_port: Option<u16>,
        cert: Option<PathBuf>,
        key: Option<PathBuf>,
        port: Option<u16>,
    },
    Configure {
        password: Option<String>,
        regen_secret: bool,
    },
}

/// Returns the application's clap [`Command`](clap::builder::Command).
pub fn cmd() -> Command {
    Command::new("pet-monitor-app")
        .about("A simple and secure pet monitor for Linux")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("configure")
                .about("Set configuration options")
                .arg(arg!(--password <PASSWORD> "The new password to set").required(false))
                .arg(arg!(--"regen-secret" "Regenerates the secret used for signing JWTs")
                    .action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("start")
                .about("Starts the server")
                .arg(arg!(-p --port <PORT> "Set the port to listen on")
                    .required(false)
                    .value_parser(value_parser!(u16)))
                .arg(arg!(--"tls-port" <PORT> "Set the port to listen on for HTTPS")
                    .required(false)
                    .value_parser(value_parser!(u16)))
                .arg(arg!(--tls <ENABLED> "Enable or disable TLS. Overrides the config file.")
                    .required(false)
                    .value_parser(value_parser!(bool)))
                .arg(arg!(--cert <CERT_PATH> "Path to an SSL certificate. Overrides the value in the config file. If the config file does not set an SSL cert key path, one must be specified in the CLI.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf))
                    .value_hint(ValueHint::FilePath))
                .arg(arg!(--key <KEY_PATH> "Path to an SSL certificate key. Overrides the value in the config file.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf))
                    .value_hint(ValueHint::FilePath))
        )
        .subcommand_required(true)
        .arg(arg!(-c --config <CONFIG> "Path to the configuration file to use.")
            .required(false)
            .value_parser(value_parser!(PathBuf))
            .value_hint(ValueHint::FilePath))
        .arg(arg!(verbosity: -v... "Log verbosity level, use 0, 1, or 2 times to set the log level to `info`, `debug`, or `trace`, respectively. This flag is overrided by the `-q` flag."))
        .arg(arg!(quiet: -q... "Silent mode, use 1 or 2 times to set the log level to `warn` or `error`, respectively. This flag overrides any use of the `-v` flag."))
}

/// Parses an iterator over CLI args into a [`Cmd`] struct.
pub fn parse_args<I, T>(args: I) -> Cmd
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = cmd().get_matches_from(args);
    Cmd {
        command: match matches.subcommand() {
            Some(("configure", matches)) => SubCmd::Configure {
                password: matches.get_one::<String>("password").map(|s| s.to_owned()),
                regen_secret: matches.get_flag("regen-secret"),
            },
            Some(("start", matches)) => SubCmd::Start {
                tls: matches.get_one::<bool>("tls").map(|x| x.to_owned()),
                tls_port: matches.get_one::<u16>("tls-port").map(|x| x.to_owned()),
                cert: matches.get_one::<PathBuf>("cert").map(|x| x.to_owned()),
                key: matches.get_one::<PathBuf>("key").map(|x| x.to_owned()),
                port: matches.get_one::<u16>("port").map(|x| x.to_owned()),
            },
            _ => unreachable!("`Command::subcommand_required` guarantees this"),
        },
        conf_path: matches.get_one::<PathBuf>("config").map(|s| s.to_owned()),
        log_level: match matches.get_count("quiet") {
            2.. => Level::Error,
            1 => Level::Warn,
            0 => match matches.get_count("verbosity") {
                0 => Level::Info,
                1 => Level::Debug,
                2.. => Level::Trace,
            },
        },
    }
}

pub async fn merge_ctx(cmd: &Cmd, mut ctx: Context) -> anyhow::Result<Context> {
    match &cmd.command {
        SubCmd::Configure {
            password,
            regen_secret,
        } => {
            let rng = SystemRandom::new();

            if let Some(pwd) = password {
                ctx.password_hash = secrets::init_password(&rng, pwd).await?;
            }

            if *regen_secret {
                ctx.jwt_secret = secrets::new_secret(&rng)?;
            }
        }
        SubCmd::Start {
            tls,
            port,
            tls_port,
            cert,
            key,
        } => {
            if let Some(port) = port {
                ctx.port = *port;
            }
            match &mut ctx.tls {
                Some(ctx_tls) if *tls != Some(false) => {
                    if let Some(tls_port) = tls_port {
                        ctx_tls.port = *tls_port;
                    }
                    if let Some(cert) = cert {
                        ctx_tls.cert = cert.clone();
                    }
                    if let Some(key) = key {
                        ctx_tls.key = key.clone();
                    }
                }
                Some(_) if *tls == Some(false) => ctx.tls = None,
                Some(_) => unreachable!(),
                None => match (tls, cert, key) {
                    (Some(tls), Some(cert), Some(key)) if *tls => {
                        ctx.tls = Some(Tls {
                            port: tls_port.unwrap_or_else(|| Tls::default().port),
                            cert: cert.clone(),
                            key: key.clone(),
                        });
                    }
                    (Some(true), _, _) => anyhow::bail!("Since the config file does not set up TLS, both a cert and key path must be specified."),
                    _ => (),
                },
            }
        }
    }
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, Arbitrary, Gen};
    use rocket::tokio;

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
            }
        }

        args
    }

    #[test]
    fn verify_cmd() {
        cmd().debug_assert();
    }

    impl Arbitrary for Cmd {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                command: SubCmd::arbitrary(g),
                conf_path: Option::<PathBuf>::arbitrary(g),
                log_level: match u32::arbitrary(g) % 5 {
                    0 => Level::Trace,
                    1 => Level::Debug,
                    2 => Level::Info,
                    3 => Level::Warn,
                    4 => Level::Error,
                    _ => panic!(),
                },
            }
        }
    }

    impl Arbitrary for SubCmd {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                Self::Configure {
                    password: Option::<String>::arbitrary(g),
                    regen_secret: bool::arbitrary(g),
                }
            } else {
                Self::Start {
                    tls: Option::<bool>::arbitrary(g),
                    tls_port: Option::<u16>::arbitrary(g),
                    cert: Option::<PathBuf>::arbitrary(g),
                    key: Option::<PathBuf>::arbitrary(g),
                    port: Option::<u16>::arbitrary(g),
                }
            }
        }
    }

    quickcheck! {
        fn qc_cmd(cmd: Cmd) -> bool {
            let args = make_args(&cmd);
            cmd == parse_args(args)
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
}
