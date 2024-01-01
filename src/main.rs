#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use uuidgen::{UUID, WellKnownUUID};

/*
301234790751488373613524883013321389239

e29fb5db-75e3-44ca-9594-b54c073c74b7
e29fb5db75e344ca9594b54c073c74b7
*/

fn main() {

    // let uuid = UUID::from_val(301234790751488373613524883013321389239);
    let uuid: UUID = "e29fb5db-75e3-44ca-9594-b54c073c74b7".try_into().unwrap();

    println!("{}", uuid.value());

    // let uuid = uuidgen::v4();
    // println!("{}", WellKnownUUID::Nil);
    // println!("{}", uuid.to_string_hex());
    // println!("{}", uuid.to_string_hex_joined());
    // println!("{}", uuid.value());
}