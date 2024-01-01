#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

fn main() {
    let value = uuidgen::v4();

    let output = format!("{}", value);

    println!("{}", output);
}