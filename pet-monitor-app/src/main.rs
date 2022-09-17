#![deny(unsafe_code)]

use config::Tls;
use ring::rand::SystemRandom;

mod cli;
mod config;
mod secrets;
mod server;
mod stream;
#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let cmd = cli::parse_args(std::env::args());

    let mut ctx = config::load(&cmd.conf_path).await?;

    match cmd.command {
        cli::SubCmd::Configure {
            password,
            regen_secret,
        } => {
            let rng = SystemRandom::new();

            if let Some(pwd) = password {
                ctx.password_hash = secrets::init_password(&rng, &pwd).await?;
                println!("Hashed new password");
            }

            if regen_secret {
                ctx.jwt_secret = secrets::new_secret(&rng)?;
                println!("Regenerated JWT signing secret");
            }

            config::store(&cmd.conf_path, &ctx).await?;
        }
        cli::SubCmd::Start {
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
                        ctx.tls = Some(Tls {
                            port: tls_port.unwrap_or_else(|| Tls::default().port),
                            cert,
                            key,
                        });
                    }
                    _ => (),
                },
            }
            server::launch(cmd.conf_path, ctx).await;
        }
    }
    Ok(())
}
