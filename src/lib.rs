#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::RngCore;
use std::fmt::{Debug, Display};
use std::iter::IntoIterator;

/*
pub fn v1() {}
pub fn v2() {}
pub fn v3(namespace: UUID, name: &str) {}
pub fn v4() {}
pub fn v5(namespace: UUID, name: &str) {}
*/

trait OctetHex<'a>
where
    Self: 'a + IntoIterator<Item = &'a u8> + Sized
{
    fn output_hex(self, f: &mut std::fmt::Formatter<'_>)  -> std::fmt::Result {
        for oct in self.into_iter() {
            write!(f, "{:02x}", oct)?;
        }

        Ok(())
    }
}

impl<'a> OctetHex<'a> for &'a [u8] {}

pub struct UUID(pub u128);

impl Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let octets = self.0.to_be_bytes();

        octets[0..=3].output_hex(f)?;
        write!(f, "-")?;
        octets[4..=5].output_hex(f)?;
        write!(f, "-")?;
        octets[6..=7].output_hex(f)?;
        write!(f, "-")?;
        octets[8..=9].output_hex(f)?;
        write!(f, "-")?;
        octets[10..=15].output_hex(f)?;

        Ok(())
    }
}

impl Debug for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUID(")?;
        <Self as Display>::fmt(self, f)?;
        write!(f, ")")?;

        Ok(())
    }
}

pub fn v4() -> UUID {
    let mut octets = [0u8; 16];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut octets);

    octets[6] = (octets[6] & 0x0f) | 0x40;
    octets[8] = (octets[8] & 0x3f) | 0x80;

    UUID(u128::from_be_bytes(octets))
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn simple_sum() {
        println!("hello");
        assert_eq!(1 + 2, 3);
    }
}
