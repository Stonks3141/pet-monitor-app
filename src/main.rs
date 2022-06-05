use pet_monitor_app::routes::*;
use rocket::fs::{relative, FileServer};
use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("client/build/")).rank(1))
        .mount("/", routes![index, stream, login])
}
