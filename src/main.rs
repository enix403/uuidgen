#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use uuidgen::UUID;

fn main() {
    let uuid = UUID::v4();

    println!("{}", uuid.to_string_hex());
    println!("{}", uuid.to_string_hex_joined());
    println!("{}", uuid.value());
}