//! This module handles command-line interactions with the application.

use crate::config::Tls;
use clap::{arg, builder::Command, value_parser};
use std::path::PathBuf;

/// A struct for command-line args
#[derive(Debug, PartialEq)]
pub struct Cmd {
    pub command: SubCmd,
    pub conf_path: Option<PathBuf>,
}

/// The CLI subcommand
#[derive(Debug, PartialEq)]
pub enum SubCmd {
    Start {
        tls: Option<Tls>,
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
        .about("A simple and secure pet monitor")
        .long_about(
            "A simple and secure pet monitor. This program is a web \
        server that handles authentication and media streaming.",
        )
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
                .arg(arg!(-p --port <PORT> ... "Set the port to listen on")
                    .required(false)
                    .value_parser(value_parser!(u16))
                    .max_values(2))
                .arg(arg!(--tls [ENABLED] "Enable or disable TLS. Overrides the config file.")
                    .required(false)
                    .value_parser(value_parser!(bool)))
                .arg(arg!(-c --cert <CERT_PATH> "Path to an SSL certificate. Overrides the value in the config file. If the config file does not set an SSL cert key path, one mus be specified in the CLI.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-k --key <KEY_PATH> "Path to an SSL certificate key. Overrides the value in the config file.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)))
        )
        .subcommand_required(true)
        .arg(
            arg!(-c --config <CONFIG> "Path to configuration file")
                .value_parser(value_parser!(PathBuf))
                .required(false),
        )
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
                tls: None,
                port: None,
            },
            _ => unreachable!("`Command::subcommand_required` guarantees this"),
        },
        conf_path: matches.get_one::<PathBuf>("config").map(|s| s.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                SubCmd::Start { tls, port } => " start".to_string(),
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
}
