use pet_monitor_app::routes::*;
use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![stream, login])
}
