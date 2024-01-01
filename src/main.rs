#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use uuidgen::{UUID, WellKnownUUID};

fn main() {
    let uuid = UUID::v5(UUID::v4(), b"foo");
    println!("{}", uuid.to_string_hex());
    println!("{}", uuid.to_string_hex_joined());
    println!("{}", uuid.value());
}