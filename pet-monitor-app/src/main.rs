#[rocket::main]
async fn main() {
    pet_monitor_app::rocket().await;
}
