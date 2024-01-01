#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::RngCore;
use std::fmt::{Debug, Display};
use std::iter::IntoIterator;

trait OctetHex<'a>
where
    Self: 'a + IntoIterator<Item = &'a u8> + Sized,
{
    fn output_hex(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for oct in self.into_iter() {
            write!(f, "{:02x}", oct)?;
        }

        Ok(())
    }
}

impl<'a> OctetHex<'a> for &'a [u8] {}

pub struct UUID(u128);

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

impl UUID {
    pub fn to_string_hex(&self) -> String {
        format!("{self}")
    }

    pub fn to_string_hex_joined(&self) -> String {
        let mut output = String::with_capacity(32);

        for oct in self.0.to_be_bytes() {
            let byte_str = format!("{:02x}", oct);
            output.push_str(&byte_str);
        }

        output
    }

    pub fn value(&self) -> u128 {
        self.0
    }
}

impl UUID {
    pub fn v4() -> UUID {
        let mut octets = [0u8; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut octets);

        octets[6] = (octets[6] & 0x0f) | 0x40;
        octets[8] = (octets[8] & 0x3f) | 0x80;

        UUID(u128::from_be_bytes(octets))
    }

    /*

    pub fn v1() {}
    pub fn v2() {}
    pub fn v3(namespace: UUID, name: &str) {}
    pub fn v4() {}
    pub fn v5(namespace: UUID, name: &str) {}

    */
}