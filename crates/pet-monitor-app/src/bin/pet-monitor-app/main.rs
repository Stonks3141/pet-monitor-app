#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use pet_monitor_app::config;
use ring::rand::{SecureRandom, SystemRandom};
use std::{
    io::{stdin, stdout},
    path::PathBuf,
};
use termion::input::TermRead;
use tokio::task::spawn_blocking;

#[cfg(not(test))]
const ARGON2_CONFIG: argon2::Config = argon2::Config {
    ad: &[],
    hash_length: 32, // bytes
    lanes: 4,
    mem_cost: 32768, // KiB
    secret: &[],
    thread_mode: argon2::ThreadMode::Parallel,
    time_cost: 8,
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
};

#[cfg(test)]
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    xflags::xflags! {
        /// A simple and secure pet monitor for Linux
        cmd pet-monitor-app {
            /// Path to the configuration file to use
            optional -c, --config path: PathBuf

            /// Print the version and exit
            cmd version {}
            /// Set the password (reads from stdin)
            cmd set-password {}
            /// Regenerate the authentication secret
            cmd regen-secret {}
            /// Start the server
            cmd start {
                /// Set the port to listen on
                optional -p, --port port: u16
                /// Disable video streaming
                optional --no-stream
            }
        }
    }

    let flags = PetMonitorApp::from_env_or_exit();

    if let PetMonitorAppCmd::Version(_) = flags.subcommand {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    env_logger::init();

    let rng = SystemRandom::new();

    let mut ctx = config::load(flags.config.clone()).await?;

    if ctx.jwt_secret == [0; 32] {
        rng.fill(&mut ctx.jwt_secret)?;
        config::store(flags.config.clone(), ctx.clone()).await?;
    }

    match flags.subcommand {
        PetMonitorAppCmd::Version(_) => unreachable!(),
        PetMonitorAppCmd::RegenSecret(_) => {
            rng.fill(&mut ctx.jwt_secret)?;
            config::store(flags.config.clone(), ctx.clone()).await?;
            eprintln!("Successfully regenerated JWT signing secret.");
        }
        PetMonitorAppCmd::SetPassword(_) => {
            eprint!("Enter password: ");
            let Some(password) = stdin().read_passwd(&mut stdout())? else {
                eprintln!("\nNo password entered.");
                std::process::exit(1);
            };
            eprintln!();
            if password.chars().count() < 3 {
                eprintln!("Password must be at least 3 characters.");
                std::process::exit(1);
            }
            let mut buf = [0u8; 16];
            rng.fill(&mut buf)?;
            ctx.password_hash = spawn_blocking(move || {
                argon2::hash_encoded(password.as_bytes(), &buf, &ARGON2_CONFIG)
            })
            .await??;
            config::store(flags.config.clone(), ctx.clone()).await?;
            eprintln!("Successfully reset password.");
        }
        PetMonitorAppCmd::Start(Start { no_stream, port }) => {
            if let Some(port) = port {
                ctx.port = port;
            }
            pet_monitor_app::start(flags.config, ctx, !no_stream).await?;
        }
    }
    Ok(())
}
