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

use ring::rand::{SecureRandom, SystemRandom};
use std::time::Instant;

/// used for finding good argon2 params
#[test]
fn argon2_time() {
    let now = Instant::now();
    {
        let rand = SystemRandom::new();
        let mut buf = [0u8; 16];
        rand.fill(&mut buf).unwrap();

        let config = argon2::Config {
            mem_cost: 8192, // KiB
            time_cost: 2,
            lanes: 4,
            variant: argon2::Variant::Argon2id,
            ..Default::default()
        };

        argon2::hash_encoded("password".as_bytes(), &buf, &config).unwrap();
    }
    let elapsed = now.elapsed();
    println!("{:?}", elapsed);
}
