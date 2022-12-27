//! This module handles command-line interactions with the application.

use crate::config::{Context, Tls};
use crate::secrets;
use clap::builder::{ArgAction, Command, ValueHint};
use clap::{arg, value_parser};
use log::Level;
use ring::rand::SystemRandom;
use std::path::PathBuf;

#[cfg(test)]
mod tests;

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
        stream: bool,
    },
    SetPassword(String),
    RegenSecret,
}

/// Returns the application's clap [`Command`](clap::builder::Command).
pub fn cmd() -> Command {
    Command::new("pet-monitor-app")
        .about("A simple and secure pet monitor for Linux.")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            arg!(-c --config <CONFIG> "Path to the configuration file to use")
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath)
                .global(true),
        )
        .arg(arg!(-v --verbose... "Increase the log level").global(true))
        .arg(arg!(-q --quiet... "Reduce the log level").global(true))
        .subcommand(
            Command::new("set-password")
                .about("Set the password")
                .arg(arg!(password: <PASSWORD> "The new password to set")),
        )
        .subcommand(Command::new("regen-secret").about("Regenerate the secret used to sign JWTs"))
        .subcommand(
            Command::new("start")
                .about("Start the server")
                .arg(
                    arg!(-p --port <PORT> "Set the port to listen on")
                        .required(false)
                        .value_parser(value_parser!(u16)),
                )
                .arg(
                    arg!(--"tls-port" <PORT> "Set the port to listen on for HTTPS")
                        .required(false)
                        .value_parser(value_parser!(u16)),
                )
                .arg(
                    arg!(--tls <ENABLED> "Enable or disable TLS.")
                        .required(false)
                        .value_parser(value_parser!(bool)),
                )
                .arg(
                    arg!(--cert <CERT_PATH> "Path to an SSL certificate")
                        .required(false)
                        .value_parser(value_parser!(PathBuf))
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    arg!(--key <KEY_PATH> "Path to an SSL certificate key")
                        .required(false)
                        .value_parser(value_parser!(PathBuf))
                        .value_hint(ValueHint::FilePath),
                )
                .arg(arg!(--"no-stream" "Disable video streaming").action(ArgAction::SetFalse)),
        )
        .subcommand_required(true)
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
            Some(("set-password", matches)) => {
                #[allow(clippy::unwrap_used)] // the "password" arg is required
                SubCmd::SetPassword(matches.get_one::<String>("password").cloned().unwrap())
            }
            Some(("regen-secret", _)) => SubCmd::RegenSecret,
            Some(("start", matches)) => SubCmd::Start {
                tls: matches.get_one::<bool>("tls").copied(),
                tls_port: matches.get_one::<u16>("tls-port").copied(),
                cert: matches.get_one::<PathBuf>("cert").cloned(),
                key: matches.get_one::<PathBuf>("key").cloned(),
                port: matches.get_one::<u16>("port").copied(),
                stream: {
                    // no-stream has a default value and is always present
                    #[allow(clippy::unwrap_used)]
                    *matches.get_one::<bool>("no-stream").unwrap()
                },
            },
            _ => unreachable!("`Command::subcommand_required` guarantees this"),
        },
        conf_path: matches.get_one::<PathBuf>("config").cloned(),
        log_level: match matches.get_count("quiet") {
            2.. => Level::Error,
            1 => Level::Warn,
            0 => match matches.get_count("verbose") {
                0 => Level::Info,
                1 => Level::Debug,
                2.. => Level::Trace,
            },
        },
    }
}

pub async fn merge_ctx(cmd: &Cmd, mut ctx: Context) -> anyhow::Result<Context> {
    match &cmd.command {
        SubCmd::RegenSecret => {
            let rng = SystemRandom::new();
            ctx.jwt_secret = secrets::new_secret(&rng)?;
        }
        SubCmd::SetPassword(password) => {
            let rng = SystemRandom::new();
            ctx.password_hash = secrets::init_password(&rng, password).await?;
        }
        SubCmd::Start {
            tls,
            port,
            tls_port,
            cert,
            key,
            ..
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
