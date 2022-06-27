use pet_monitor_app::routes::*;
use pet_monitor_app::secrets;
use ring::rand::SystemRandom;
use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    secrets::RAND.set(SystemRandom::new()).unwrap(); // infallible, the value can't be set at this point

    secrets::PASSWORD_HASH
        .set(secrets::init_pwd().expect("Failed to initialize password."))
        .unwrap(); // infallible

    secrets::SECRET
        .set(secrets::init_secret().expect("Failed to initialize JWT secret."))
        .unwrap(); // infallible

    rocket::build().mount("/", routes![login, stream])
}
