use clap::{arg, builder::Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Options {
    pub regen_secret: bool,
    pub password: Option<String>,
    pub conf_path: Option<PathBuf>,
}

pub fn cmd() -> Command<'static> {
    Command::new("pet-monitor-app")
        .about("A simple and secure pet monitor")
        .long_about(
            "A simple and secure pet monitor. This program is a web \
        server that handles authentication and media streaming, intended for \
        use as a pet monitor.",
        )
        .author("Sam Nystrom")
        .version(env!("CARGO_PKG_VERSION"))
        .args(&[
            arg!(   --password [PASSWORD] "Reset the password").min_values(1),
            arg!(   --"regen-secret"      "Regenerate the JWT secret"),
            arg!(-c --config [CONFIG]     "Path to configuration file").min_values(1),
        ])
}

pub fn parse_args<I, T>(args: I) -> Options
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = cmd().get_matches_from(args);

    Options {
        regen_secret: matches.contains_id("regen-secret"),
        password: matches.get_one("password").cloned(),
        conf_path: matches
            .get_one::<String>("config")
            .cloned()
            .map(PathBuf::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cmd() {
        cmd().debug_assert();
    }
}
