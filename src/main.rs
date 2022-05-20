use pet_monitor_app::routes;
use rocket::fs::{FileServer, relative};
use rocket::{launch, routes};
//use v4l_streamer::Frames;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("client/build/")).rank(1))
        .mount("/", routes![routes::index])
}
