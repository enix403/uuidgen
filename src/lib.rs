#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::RngCore;
use std::fmt::{Debug, Display};

/*
pub fn v1() {}
pub fn v2() {}
pub fn v3(namespace: UUID, name: &str) {}
pub fn v4() {}
pub fn v5(namespace: UUID, name: &str) {}
*/

pub struct UUID(pub u128);

impl Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let octets = self.0.to_be_bytes();

        let mut write_range = |f: &mut std::fmt::Formatter<'_>, a: usize, b: usize| {
            for oct in &octets[a..=b] {
                write!(f, "{:02x}", oct)?;
            }

            Ok(())
        };

        write_range(f, 0, 3)?;
        write!(f, "-")?;
        write_range(f, 4, 5)?;
        write!(f, "-")?;
        write_range(f, 6, 7)?;
        write!(f, "-")?;
        write_range(f, 8, 9)?;
        write!(f, "-")?;
        write_range(f, 10, 15)?;

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
