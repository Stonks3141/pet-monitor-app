use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cmd {
    /// Path to the configuration file to use
    #[arg(short, long = "config", value_name = "CONFIG")]
    pub conf_path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: SubCmd,
}

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
    /// Set the password (reads password from stdin)
    SetPassword,
    /// Regenerate the secret used to sign JWTs
    RegenSecret,
}
