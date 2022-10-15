#![deny(unsafe_code)]

use log::{info, Level};
use rocket::config::LogLevel;

mod bmff;
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
    let ctx = cli::merge_ctx(&cmd, ctx).await?;

    match cmd.command {
        cli::SubCmd::Configure { .. } => {
            config::store(&cmd.conf_path, &ctx).await?;
            info!("Successfully updated configuration!");
        }
        cli::SubCmd::Start { .. } => {
            let level = match cmd.log_level {
                Level::Error => LogLevel::Critical,
                Level::Warn => LogLevel::Critical,
                Level::Info => LogLevel::Normal,
                Level::Debug => LogLevel::Debug,
                Level::Trace => LogLevel::Debug,
            };
            server::launch(cmd.conf_path, ctx, level).await?;
        }
    }
    Ok(())
}
