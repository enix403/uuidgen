#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use uuidgen::{UUID, wellknown};

fn main() {
    let uuid = UUID::v5(b"foo", Some(wellknown::NS_DNS));
    println!("{}", uuid.to_string_hex());
    // println!("{}", uuid.to_string_hex_joined());
    // println!("{}", uuid.value());

}