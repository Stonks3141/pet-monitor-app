//! This module handles command-line interactions with the application.

use crate::config::{Context, Tls};
use crate::secrets;
use clap::{arg, builder::Command, value_parser};
use ring::rand::SystemRandom;
use std::path::PathBuf;

/// A struct for command-line args
#[derive(Debug, PartialEq, Eq)]
pub struct Cmd {
    pub command: SubCmd,
    pub conf_path: Option<PathBuf>,
}

/// The CLI subcommand
#[derive(Debug, PartialEq, Eq)]
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
pub fn cmd() -> Command<'static> {
    Command::new("pet-monitor-app")
        .about("A simple and secure pet monitor for Linux")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("configure")
                .about("Set configuration options")
                .arg(arg!(--password <PASSWORD> "The new password to set").required(false))
                .arg(arg!(--"regen-secret" "Regenerates the secret used for signing JWTs")),
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
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(--key <KEY_PATH> "Path to an SSL certificate key. Overrides the value in the config file.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)))
        )
        .subcommand_required(true)
        .arg(arg!(-c --config <CONFIG> "Path to configuration file")
            .value_parser(value_parser!(PathBuf))
            .required(false))
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
                regen_secret: matches.is_present("regen-secret"),
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
                    (None, Some(_), Some(_)) => (),
                    _ => anyhow::bail!("Since the config file does not set up TLS, both a cert and key path must be specified."),
                },
            }
        }
    }
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::tokio;

    fn make_args(cmd: &Cmd) -> String {
        format!(
            "pet-monitor-app{}{}",
            if let Some(conf_path) = &cmd.conf_path {
                format!(
                    " --config {}",
                    conf_path.clone().into_os_string().into_string().unwrap(),
                )
            } else {
                String::new()
            },
            match &cmd.command {
                SubCmd::Configure {
                    password,
                    regen_secret,
                } => format!(
                    " configure{}{}",
                    if *regen_secret { " --regen-secret" } else { "" },
                    if let Some(password) = password {
                        format!(" --password {}", password)
                    } else {
                        String::new()
                    },
                ),
                SubCmd::Start {
                    tls,
                    port,
                    tls_port,
                    cert,
                    key,
                } => format!(
                    " start{}{}{}{}{}",
                    if let Some(tls) = tls {
                        format!(" --tls {}", tls)
                    } else {
                        String::new()
                    },
                    if let Some(port) = port {
                        format!(" --port {}", port)
                    } else {
                        String::new()
                    },
                    if let Some(tls_port) = tls_port {
                        format!(" --tls-port {}", tls_port)
                    } else {
                        String::new()
                    },
                    if let Some(cert) = cert {
                        format!(
                            " --cert {}",
                            cert.to_owned().into_os_string().into_string().unwrap()
                        )
                    } else {
                        String::new()
                    },
                    if let Some(key) = key {
                        format!(
                            " --key {}",
                            key.to_owned().into_os_string().into_string().unwrap()
                        )
                    } else {
                        String::new()
                    },
                ),
            },
        )
        .to_string()
    }

    #[test]
    fn verify_cmd() {
        cmd().debug_assert();
    }

    #[test]
    fn cmd_configure() {
        let cmd = Cmd {
            command: SubCmd::Configure {
                regen_secret: true,
                password: Some("password".to_string()),
            },
            conf_path: Some(PathBuf::from("./pet-monitor-app.toml")),
        };
        let args = make_args(&cmd);
        assert_eq!(cmd, parse_args(args.split(' ')));
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
        };

        let ctx = Context::default();
        let ctx = merge_ctx(&cmd, ctx.clone()).await?;

        assert!(crate::secrets::validate(password, &ctx.password_hash)
            .await
            .unwrap());

        Ok(())
    }
}
