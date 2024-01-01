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
    pub fn from_val(value: u128) -> Self {
        Self(value)
    }

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

impl TryFrom<&str> for UUID {
    type Error = ();
    
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let mut intval = 0u128;
            let mut consumed = 0;

            for s in value.chars() {
                let as_int: u128 = match s {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    '5' => 5,
                    '6' => 6,
                    '7' => 7,
                    '8' => 8,
                    '9' => 9,
                    'a' | 'A' => 10,
                    'b' | 'B' => 11,
                    'c' | 'C' => 12,
                    'd' | 'D' => 13,
                    'e' | 'E' => 14,
                    'f' | 'F' => 15,
                    '-' => continue,
                    _ => return Err(()),
                };

                intval = intval << 4 | as_int;
                consumed += 1;

                if consumed == 32 {
                    break;
                }
            }

            if consumed == 32 {
                Ok(UUID(intval))
            } else {
                Err(())
            }
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

#[non_exhaustive]
pub struct WellKnownUUID {}

#[allow(non_upper_case_globals)]
impl WellKnownUUID {
    pub const Nil: UUID = UUID(0);
}
