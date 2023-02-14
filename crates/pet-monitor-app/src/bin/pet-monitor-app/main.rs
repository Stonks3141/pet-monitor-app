#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use color_eyre::eyre;
use pet_monitor_app::config;
use ring::rand::{SecureRandom, SystemRandom};
use std::path::PathBuf;
use tokio::task::spawn_blocking;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    xflags::xflags! {
        cmd pet-monitor-app {
            /// Print version
            optional -V, --version
            /// Path to the configuration file to use
            optional -c, --config path: PathBuf
            /// Regenerate the secret used for signing JWTs
            cmd regen-secret {}
            /// Set the password
            cmd set-password {
                required password: String
            }
            /// Start the server
            cmd start {
                /// Enable or disable TLS
                optional --tls tls: bool
                /// Set the port to listen on for HTTPS
                optional --tls-port tls_port: u16
                /// Path to an SSL certificate
                optional --cert cert: PathBuf
                /// Path to an SSL certificate key
                optional --key cert: PathBuf
                /// Set the port to listen on
                optional -p, --port port: u16
                /// Disable video streaming
                optional --no-stream
            }
        }
    };
    let cmd = PetMonitorApp::from_env_or_exit();

    if cmd.version {
        println!("pet-monitor-app {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_ansi(true)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default())
        .init();

    let rng = SystemRandom::new();

    let mut ctx = config::load(cmd.config.clone()).await?;

    if ctx.jwt_secret == [0; 32] {
        rng.fill(&mut ctx.jwt_secret)?;
        config::store(cmd.config.clone(), ctx.clone()).await?;
    }

    match cmd.subcommand {
        PetMonitorAppCmd::RegenSecret(_) => {
            rng.fill(&mut ctx.jwt_secret)?;
            config::store(cmd.config.clone(), ctx.clone()).await?;
            println!("Successfully regenerated JWT signing secret.");
        }
        PetMonitorAppCmd::SetPassword(SetPassword { password }) => {
            let mut buf = [0u8; 16];
            rng.fill(&mut buf)?;
            ctx.password_hash = spawn_blocking(move || {
                argon2::hash_encoded(password.as_bytes(), &buf, &ARGON2_CONFIG)
            })
            .await??;
            config::store(cmd.config.clone(), ctx.clone()).await?;
            println!("Successfully reset password.");
        }
        PetMonitorAppCmd::Start(Start {
            no_stream,
            tls,
            port,
            tls_port,
            cert,
            key,
        }) => {
            if let Some(port) = port {
                ctx.port = port;
            }
            match &mut ctx.tls {
                Some(ctx_tls) if tls != Some(false) => {
                    if let Some(tls_port) = tls_port {
                        ctx_tls.port = tls_port;
                    }
                    if let Some(cert) = cert {
                        ctx_tls.cert = cert;
                    }
                    if let Some(key) = key {
                        ctx_tls.key = key;
                    }
                }
                Some(_) if tls == Some(false) => ctx.tls = None,
                Some(_) => unreachable!(),
                None => match (tls, cert, key) {
                    (Some(tls), Some(cert), Some(key)) if tls => {
                        ctx.tls = Some(config::Tls {
                            port: tls_port.unwrap_or(8443),
                            cert,
                            key,
                        });
                    }
                    (Some(true), _, _) => eyre::bail!(
                        "Since the config file does not set up TLS, both a cert and key path must be specified."
                    ),
                    _ => (),
                },
            }
            pet_monitor_app::start(cmd.config, ctx, !no_stream).await?;
        }
    }
    Ok(())
}
