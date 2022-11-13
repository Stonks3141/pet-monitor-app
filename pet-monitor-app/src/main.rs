#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::dbg_macro)]
#![deny(non_ascii_idents)]

#[cfg(not(target_os = "linux"))]
compile_error!("Linux is required for V4L2");

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
    let ctx = cli::merge_ctx(&cmd, ctx).await?;

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
