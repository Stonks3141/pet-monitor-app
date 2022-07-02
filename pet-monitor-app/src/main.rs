// Pet Montitor App
// Copyright (C) 2022  Samuel Nystrom
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
