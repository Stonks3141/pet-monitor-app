//! This module handles command-line interactions with the application.

use crate::config::{Context, Tls};
use crate::secrets;
use clap::{Parser, Subcommand};
use ring::rand::SystemRandom;
use std::path::PathBuf;

/// A struct for command-line args
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cmd {
    /// Path to the configuration file to use
    #[arg(short, long = "config", value_name = "CONFIG")]
    pub conf_path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: SubCmd,
}

/// The CLI subcommand
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum SubCmd {
    /// Start the server
    Start {
        /// Enable or disable TLS
        #[arg(long)]
        tls: Option<bool>,
        /// Set the port to listen on for HTTPS
        #[arg(long)]
        tls_port: Option<u16>,
        /// Path to an SSL certificate
        #[arg(long)]
        cert: Option<PathBuf>,
        /// Path to an SSL certificate key
        #[arg(long)]
        key: Option<PathBuf>,
        /// Set the port to listen on
        #[arg(short, long)]
        port: Option<u16>,
        /// Disable video streaming
        #[arg(long = "no-stream", action = clap::ArgAction::SetFalse)]
        stream: bool,
    },
    /// Set the password
    SetPassword {
        /// The new password to set
        password: String,
    },
    /// Regenerate the secret used to sign JWTs
    RegenSecret,
}

pub async fn merge_ctx(cmd: &Cmd, mut ctx: Context) -> anyhow::Result<Context> {
    match &cmd.command {
        SubCmd::RegenSecret => {
            let rng = SystemRandom::new();
            ctx.jwt_secret = secrets::new_secret(&rng)?;
        }
        SubCmd::SetPassword { password } => {
            let rng = SystemRandom::new();
            ctx.password_hash = secrets::init_password(&rng, password).await?;
        }
        SubCmd::Start {
            tls,
            port,
            tls_port,
            cert,
            key,
            ..
        } => {
            if let Some(port) = port {
                ctx.port = *port;
            }
            match &mut ctx.tls {
                Some(ctx_tls) if *tls != Some(false) => {
                    if let Some(tls_port) = tls_port {
                        ctx_tls.port = *tls_port;
                    }
                    if let Some(cert) = cert {
                        ctx_tls.cert = cert.clone();
                    }
                    if let Some(key) = key {
                        ctx_tls.key = key.clone();
                    }
                }
                Some(_) if *tls == Some(false) => ctx.tls = None,
                Some(_) => unreachable!(),
                None => match (tls, cert, key) {
                    (Some(tls), Some(cert), Some(key)) if *tls => {
                        ctx.tls = Some(Tls {
                            port: tls_port.unwrap_or_else(|| Tls::default().port),
                            cert: cert.clone(),
                            key: key.clone(),
                        });
                    }
                    (Some(true), _, _) => anyhow::bail!("Since the config file does not set up TLS, both a cert and key path must be specified."),
                    _ => (),
                },
            }
        }
    }
    Ok(ctx)
}
