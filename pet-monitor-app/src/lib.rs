//! This crate provides a web server for [pet-monitor-app](https://github.com/Stonks3141/pet-monitor-app).
//!
//! The release binary should be run in a Docker container or have access to `/var/local`.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::private_doc_tests)]

pub mod auth;
pub mod routes;
pub mod secrets;
mod stream;
#[cfg(test)]
mod tests;
