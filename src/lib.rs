#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::RngCore;
use std::convert::AsRef;
use std::fmt::{Debug, Display};
use std::iter::IntoIterator;

use digest::Digest;
use md5::Md5;
use sha1::Sha1;

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

#[derive(PartialEq, Eq)]
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

    #[inline(always)]
    fn finalize_octets(mut octets: [u8; 16], version: u8) -> Self {
        octets[6] = (octets[6] & 0x0f) | (version << 4);
        octets[8] = (octets[8] & 0x3f) | 0x80;

        UUID(u128::from_be_bytes(octets))
    }

    fn hash_based_uuid<D: Digest>(
        mut hasher: D,
        namespace: UUID,
        name: &[u8],
        version: u8,
    ) -> Self {
        hasher.update(namespace.0.to_be_bytes());
        hasher.update(name);

        let hash = hasher.finalize();
        let hash = &hash[..];

        let mut octets = [0u8; 16];

        octets.copy_from_slice(&hash[0..=15]);
        Self::finalize_octets(octets, version)
    }

    pub fn v3(namespace: UUID, name: &[u8]) -> Self {
        Self::hash_based_uuid(<Md5 as Digest>::new(), namespace, name, 3)
    }

    pub fn v4() -> UUID {
        let mut octets = [0u8; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut octets);

        Self::finalize_octets(octets, 4)
    }

    pub fn v5(namespace: UUID, name: &[u8]) -> Self {
        Self::hash_based_uuid(<Sha1 as Digest>::new(), namespace, name, 5)
    }

    pub fn parse<T: AsRef<str>>(value: T) -> Result<Self, ()> {
        // Parses the following formats:
        //      8-4-4-4-12 format:
        //          aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa
        //
        //      32-length hex string format:
        //          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        //
        //      32-length hex string format with 0x or 0X prefix:
        //          0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        //          0Xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

        let mut value = value.as_ref();

        if value.starts_with("0x") {
            value = value.strip_prefix("0x").unwrap();
        } else if value.starts_with("0X") {
            value = value.strip_prefix("0X").unwrap();
        }

        let mut intval = 0u128;
        let mut consumed = 0;

        for (i, s) in value.chars().enumerate() {
            let as_int: u128 = match (i, s) {
                (_, '0') => 0,
                (_, '1') => 1,
                (_, '2') => 2,
                (_, '3') => 3,
                (_, '4') => 4,
                (_, '5') => 5,
                (_, '6') => 6,
                (_, '7') => 7,
                (_, '8') => 8,
                (_, '9') => 9,
                (_, 'a' | 'A') => 10,
                (_, 'b' | 'B') => 11,
                (_, 'c' | 'C') => 12,
                (_, 'd' | 'D') => 13,
                (_, 'e' | 'E') => 14,
                (_, 'f' | 'F') => 15,

                // Dashes are only allowed at these indices
                (8 | 13 | 18 | 23, '-') => continue,
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

impl TryFrom<&str> for UUID {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        UUID::parse(value)
    }
}

impl TryFrom<String> for UUID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        UUID::parse(value)
    }
}

#[non_exhaustive]
pub struct WellKnownUUID {}

#[allow(non_upper_case_globals)]
impl WellKnownUUID {
    pub const Nil: UUID = UUID(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v4_version() {
        let uuid = UUID::v4();
        assert_eq!(uuid.0.to_be_bytes()[6] & 0xf0, 0x40);
    }

    #[test]
    fn test_format() {
        let uuid = UUID::from_val(339909213143343215632204095398962575718);

        assert_eq!(format!("{uuid}"), "ffb82219-2be8-4961-8c83-2163e1b4b966");
        assert_eq!(uuid.to_string_hex(), "ffb82219-2be8-4961-8c83-2163e1b4b966");
        assert_eq!(
            uuid.to_string_hex_joined(),
            "ffb822192be849618c832163e1b4b966"
        );
    }

    #[test]
    fn test_parse() {
        let uuid = UUID::v4();

        assert_eq!(uuid, uuid.to_string_hex().try_into().unwrap());
        assert_eq!(uuid, uuid.to_string_hex_joined().try_into().unwrap());
        assert_eq!(uuid, UUID::from_val(uuid.value()));
    }
}
