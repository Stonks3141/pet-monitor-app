#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

mod cli;

use clap::Parser;
use pet_monitor_app::config;
use ring::rand::{SecureRandom, SystemRandom};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = cli::Cmd::parse();
    tracing_subscriber::fmt::init();

    let rng = SystemRandom::new();

    let mut ctx = config::load(cmd.conf_path.clone()).await?;

    if ctx.jwt_secret == [0; 32] {
        rng.fill(&mut ctx.jwt_secret)?;
        config::store(cmd.conf_path.clone(), ctx.clone()).await?;
    }

    match cmd.command {
        cli::SubCmd::RegenSecret => {
            rng.fill(&mut ctx.jwt_secret)?;
            config::store(cmd.conf_path.clone(), ctx.clone()).await?;
            println!("Successfully regenerated JWT signing secret.");
        }
        cli::SubCmd::SetPassword { password } => {
            let mut buf = [0u8; 16];
            rng.fill(&mut buf)?;
            ctx.password_hash = spawn_blocking(move || {
                argon2::hash_encoded(password.as_bytes(), &buf, &ARGON2_CONFIG)
            })
            .await??;
            config::store(cmd.conf_path.clone(), ctx.clone()).await?;
            println!("Successfully reset password.");
        }
        cli::SubCmd::Start {
            stream,
            tls,
            port,
            tls_port,
            cert,
            key,
        } => {
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
                    (Some(true), _, _) => anyhow::bail!(
                        "Since the config file does not set up TLS, both a cert and key path must be specified."
                    ),
                    _ => (),
                },
            }
            pet_monitor_app::start(cmd.conf_path, ctx, stream).await?;
        }
    }
    Ok(())
}
