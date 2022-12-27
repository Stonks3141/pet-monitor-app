#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use log::{info, Level};
use rocket::config::LogLevel;

mod cli;
mod config;
mod secrets;
mod server;
#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let cmd = cli::parse_args(std::env::args());
    simple_logger::init_with_level(cmd.log_level)?;

    let ctx = config::load(&cmd.conf_path).await?;
    let mut ctx = cli::merge_ctx(&cmd, ctx).await?;

    if ctx.jwt_secret == [0; 32] {
        let mut rng = ring::rand::SystemRandom::new();
        ctx.jwt_secret = secrets::new_secret(&mut rng)?;
        config::store(&cmd.conf_path, &ctx).await?;
    }

    match cmd.command {
        cli::SubCmd::Configure { .. } => {
            config::store(&cmd.conf_path, &ctx).await?;
            info!("Successfully updated configuration!");
        }
        cli::SubCmd::Start { stream, .. } => {
            let level = match cmd.log_level {
                Level::Error | Level::Warn => LogLevel::Critical,
                Level::Info => LogLevel::Normal,
                Level::Debug | Level::Trace => LogLevel::Debug,
            };
            server::launch(cmd.conf_path, ctx, level, stream).await?;
        }
    }
    Ok(())
}
