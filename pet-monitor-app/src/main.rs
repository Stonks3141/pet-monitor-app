#[rocket::launch]
async fn rocket() -> _ {
    pet_monitor_app::rocket().await
}
