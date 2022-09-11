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
                    if *regen_secret {
                        " --regen-secret"
                    } else {
                        ""
                    },
                    if let Some(password) = password {
                        format!(" --password {}", password)
                    } else {
                        String::new()
                    },
                ),
                SubCmd::Start => " start".to_string(),
            },
        )
        .to_string()
    }

    #[test]
    fn verify_cmd() {
        cmd().debug_assert();
    }

    proptest! {
        #[test]
        fn proptest_cmd_configure(regen_secret: bool, password: Option<String>, conf_path: Option<String>) {
            let cmd = Cmd {
                command: SubCmd::Configure { regen_secret, password: password.clone() },
                conf_path: conf_path.map(|p| PathBuf::from(p)),
            };
            let args = make_args(&cmd);
            assert_eq!(cmd, parse_args(args.split(' ')));
        }
    }
}
