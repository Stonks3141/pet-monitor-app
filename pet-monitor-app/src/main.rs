use pet_monitor_app::routes::*;
use pet_monitor_app::secrets;
use ring::rand::SystemRandom;
use rocket::{launch, routes};

#[launch]
async fn rocket() -> _ {
    let rng = SystemRandom::new();
    let pwd = secrets::Password::new(&rng)
        .await
        .expect("Failed to initialize password.");
    let secret = secrets::Secret::new(&rng)
        .await
        .expect("Failed to initialize JWT secret.");

    rocket::build()
        .mount("/", routes![login, verify])
        .manage(pwd)
        .manage(secret)
        .manage(rng)
}
