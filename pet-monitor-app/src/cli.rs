use clap::{arg, builder::Command, value_parser};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Cmd {
    pub command: SubCmd,
    pub conf_path: Option<PathBuf>,
}

#[derive(Debug, PartialEq)]
pub enum SubCmd {
    Start,
    Configure {
        password: Option<String>,
        regen_secret: bool,
    },
}

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
            Command::new("start").about("Starts the server").arg(
                arg!(-p --port <PORT> "Set the port to listen on")
                    .value_parser(value_parser!(u16))
                    .required(false),
            ),
        )
        .subcommand_required(true)
        .arg(
            arg!(-c --config <CONFIG> "Path to configuration file")
                .value_parser(value_parser!(PathBuf))
                .required(false),
        )
}

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
            Some(("start", _)) => SubCmd::Start,
            _ => panic!("subcommand required"),
        },
        conf_path: matches.get_one::<PathBuf>("config").map(|s| s.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn verify_cmd() {
        cmd().debug_assert();
    }

    proptest! {
        #[test]
        fn test_cmd_configure(regen_secret: bool, password: Option<String>) {
            let cmd = SubCmd::Configure { regen_secret, password: password.clone() };
            let mut args = vec!["pet-monitor-app".to_string(), "configure".to_string()];
            if regen_secret {
                args.push("--regen-secret".to_string());
            }
            if let Some(password) = password {
                args.push("--password".to_string());
                args.push(password);
            }
            assert_eq!(cmd, parse_args(args).command);
        }
    }
}
