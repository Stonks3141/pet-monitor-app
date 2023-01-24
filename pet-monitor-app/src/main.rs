#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use clap::Parser;
use pet_monitor_app::{cli, config, secrets, server};
use ring::rand::SystemRandom;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = cli::Cmd::parse();
    simple_logger::init_with_env()?;

    let rng = SystemRandom::new();

    let mut ctx = config::load(cmd.conf_path.clone()).await?;

    if ctx.jwt_secret == [0; 32] {
        ctx.jwt_secret = secrets::new_secret(&rng)?;
        config::store(cmd.conf_path.clone(), ctx.clone()).await?;
    }

    match cmd.command {
        cli::SubCmd::RegenSecret => {
            ctx.jwt_secret = secrets::new_secret(&rng)?;
            config::store(cmd.conf_path.clone(), ctx.clone()).await?;
            println!("Successfully regenerated JWT signing secret.");
        }
        cli::SubCmd::SetPassword { password } => {
            ctx.password_hash = secrets::init_password(&rng, &password).await?;
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
                    (Some(true), _, _) => anyhow::bail!("Since the config file does not set up TLS, both a cert and key path must be specified."),
                    _ => (),
                },
            }
            server::start(cmd.conf_path, ctx, stream).await?;
        }
    }
    Ok(())
}
