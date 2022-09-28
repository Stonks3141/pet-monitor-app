#![deny(unsafe_code)]

mod bmff;
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

    let ctx = config::load(&cmd.conf_path).await?;
    let ctx = cli::merge_ctx(&cmd, ctx).await?;

    match cmd.command {
        cli::SubCmd::Configure { .. } => {
            config::store(&cmd.conf_path, &ctx).await?;
        }
        cli::SubCmd::Start { .. } => {
            server::launch(cmd.conf_path, ctx).await;
        }
    }
    Ok(())
}
