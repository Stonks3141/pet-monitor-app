use clap::{arg, builder::Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Options {
    pub regen_secret: bool,
    pub password: Option<String>,
    pub conf_path: Option<PathBuf>,
}

pub fn parse_args() -> Options {
    let matches = Command::new("pet-monitor-app")
        .about("A simple and secure pet monitor")
        .long_about(
            "A simple and secure pet monitor. This program is a web \
        server that handles authentication and media streaming, intended for \
        use as a pet monitor.",
        )
        .author("Sam Nystrom")
        .version("0.1.0")
        .args(&[
            arg!(   --password [PASSWORD] "Reset the password").min_values(1),
            arg!(   --"regen-secret"      "Regenerate the JWT secret"),
            arg!(-c --config [CONFIG]     "Path to configuration file").min_values(1),
        ])
        .get_matches();

    Options {
        regen_secret: matches.contains_id("regen-secret"),
        password: matches.get_one("password").cloned(),
        conf_path: matches
            .get_one::<String>("config")
            .cloned()
            .map(PathBuf::from),
    }
}
