use pet_monitor_app::routes::*;
use pet_monitor_app::secrets;
use ring::rand::SystemRandom;
use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    let rng = SystemRandom::new();
    let pwd = secrets::Password::new(&rng).expect("Failed to initialize password.");
    let secret = secrets::Secret::new(&rng).expect("Failed to initialize JWT secret.");

    rocket::build()
        .mount("/", routes![login, stream])
        .manage(pwd)
        .manage(secret)
        .manage(rng)
}
