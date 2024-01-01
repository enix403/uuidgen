#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use uuidgen::WellKnownUUID;

fn main() {
    let uuid = uuidgen::v4();

    println!("{}", WellKnownUUID::Nil);

    println!("{}", uuid.to_string_hex());
    println!("{}", uuid.to_string_hex_joined());
    println!("{}", uuid.value());
}