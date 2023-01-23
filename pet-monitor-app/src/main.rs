#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

use clap::Parser;

mod cli;
mod config;
mod secrets;
mod server;
#[cfg(test)]
mod tests;

#[tokio::main]
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
            println!("Successfully updated configuration!");
        }
        cli::SubCmd::Start { stream, .. } => {
            server::start(cmd.conf_path, ctx, stream).await?;
        }
    }
    Ok(())
}
