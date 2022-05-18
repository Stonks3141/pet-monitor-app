//use pet_monitor_app::routes;
//use rocket::fs::{FileServer, relative};
//use rocket::{launch, routes};
use v4l_streamer::Frames;
use std::io::Write;
use std::fs;

//#[launch]
fn main() {//-> rocket::Rocket<rocket::Build> {
    let frames = Frames::new((1, 30), (1280, 720), *b"MJPG");

    frames.into_iter().enumerate().take(10)
        .for_each(|(i, frame)| {
            let mut file = fs::File::create(&format!("{}.jpg", i)).unwrap();
            file.write_all(&frame[..]).unwrap();
            file.flush().unwrap();
        });

    //rocket::build()
    //    .mount("/", FileServer::from(relative!("static/")).rank(1))
    //    .mount("/", routes![routes::index])
}
