//! This crate provides a web server for [pet-monitor-app](https://github.com/Stonks3141/pet-monitor-app).
//! 
//! The binary should be run in a Docker container or have access to `/var/local`.

pub mod auth;
pub mod routes;
pub mod secrets;
#[cfg(test)]
mod tests;
