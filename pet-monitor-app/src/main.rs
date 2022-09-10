#![deny(unsafe_code)]

use config::Context;
use human_panic::setup_panic;
use ring::rand::SystemRandom;

mod cli;
mod config;
mod server;
mod secrets;
mod stream;
#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() {
    setup_panic!();

    let cmd = cli::parse_args(std::env::args());

    let mut ctx: Context = if let Some(path) = &cmd.conf_path {
        confy::load_path(&path).expect("Failed to load configuration file")
    } else {
        confy::load("pet-monitor-app").expect("Failed to load configuration file")
    };

    match cmd.command {
        cli::SubCmd::Configure { password, regen_secret } => {
            let rng = SystemRandom::new();

            if let Some(pwd) = password {
                ctx.password_hash = secrets::init_password(&rng, &pwd).unwrap();
                println!("Hashed new password");
            }

            if regen_secret {
                ctx.jwt_secret = secrets::new_secret(&rng).unwrap();
                println!("Regenerated JWT signing secret");
            }

            if let Some(path) = &cmd.conf_path {
                confy::store_path(&path, &ctx).expect("Failed to load configuration file")
            } else {
                confy::store("pet-monitor-app", &ctx).expect("Failed to load configuration file")
            };
        }
        cli::SubCmd::Start => server::rocket(cmd.conf_path, ctx).await
    }
}
