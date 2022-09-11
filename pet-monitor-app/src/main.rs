#![deny(unsafe_code)]

use ring::rand::SystemRandom;
use anyhow::Context;

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

    let mut ctx: config::Context = if let Some(path) = &cmd.conf_path {
        confy::load_path(&path).context("Failed to load configuration file")?
    } else {
        confy::load("pet-monitor-app").context("Failed to load configuration file")?
    };

    match cmd.command {
        cli::SubCmd::Configure {
            password,
            regen_secret,
        } => {
            let rng = SystemRandom::new();

            if let Some(pwd) = password {
                ctx.password_hash = secrets::init_password(&rng, &pwd)?;
                println!("Hashed new password");
            }

            if regen_secret {
                ctx.jwt_secret = secrets::new_secret(&rng)?;
                println!("Regenerated JWT signing secret");
            }

            if let Some(path) = &cmd.conf_path {
                confy::store_path(&path, &ctx).context("Failed to load configuration file")?;
            } else {
                confy::store("pet-monitor-app", &ctx).context("Failed to load configuration file")?;
            }
        }
        cli::SubCmd::Start => server::rocket(cmd.conf_path, ctx).await,
    }
    Ok(())
}
