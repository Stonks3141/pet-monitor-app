use server::api::routes;
use rocket::fs::{FileServer, relative};
use rocket::{launch, routes};

#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", FileServer::from(relative!("../client/dist")).rank(1))
        .mount("/", routes![routes::index])
}
