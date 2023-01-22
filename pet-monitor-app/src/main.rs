#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use clap::Parser;
use log::Level;
use rocket::config::LogLevel;

mod cli;
mod config;
mod secrets;
mod server;
#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let cmd = cli::Cmd::parse();
    simple_logger::init_with_env()?;

    let ctx = config::load(&cmd.conf_path).await?;
    let mut ctx = cli::merge_ctx(&cmd, ctx).await?;

    if ctx.jwt_secret == [0; 32] {
        let rng = ring::rand::SystemRandom::new();
        ctx.jwt_secret = secrets::new_secret(&rng)?;
        config::store(&cmd.conf_path, &ctx).await?;
    }

    match cmd.command {
        cli::SubCmd::SetPassword { .. } | cli::SubCmd::RegenSecret => {
            config::store(&cmd.conf_path, &ctx).await?;
            log::info!("Successfully updated configuration!");
        }
        cli::SubCmd::Start { stream, .. } => {
            let level = match log::max_level().to_level().unwrap_or(Level::Error) {
                Level::Error | Level::Warn => LogLevel::Critical,
                Level::Info => LogLevel::Normal,
                Level::Debug | Level::Trace => LogLevel::Debug,
            };
            server::launch(cmd.conf_path, ctx, level, stream).await?;
        }
    }
    Ok(())
}
