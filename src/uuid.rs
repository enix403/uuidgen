#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::convert::AsRef;
use std::fmt::{Debug, Display};
use std::iter::IntoIterator;
use std::collections::HashMap;

use rand::RngCore;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;

use phf::phf_map;

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
pub struct Uuid(u128);

impl Display for Uuid {
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

impl Debug for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uuid(")?;
        <Self as Display>::fmt(self, f)?;
        write!(f, ")")?;

        Ok(())
    }
}

static HEX_TO_INT_TBL: phf::Map<char, u8> = phf_map! {
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

    'a' => 10,
    'b' => 11,
    'c' => 12,
    'd' => 13,
    'e' => 14,
    'f' => 15,

    'A' => 10,
    'B' => 11,
    'C' => 12,
    'D' => 13,
    'E' => 14,
    'F' => 15,
};

pub(crate) type Octets = [u8; 16];

impl Uuid {
    pub const fn from_value(value: u128) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub(crate) fn from_octets(mut octets: Octets, version_hi: u8) -> Self {
        octets[6] = (octets[6] & 0x0f) | (version_hi << 4);
        octets[8] = (octets[8] & 0x3f) | 0x80;

        Uuid(u128::from_be_bytes(octets))
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
                // Dashes are only allowed at these indices
                (8 | 13 | 18 | 23, '-') => continue,

                // Other characters
                (_, s) => match HEX_TO_INT_TBL.get(&s) {
                    Some(&val) => val as u128,
                    None => return Err(())
                }
            };

            intval = intval << 4 | as_int;
            consumed += 1;

            if consumed == 32 {
                break;
            }
        }

        if consumed == 32 {
            Ok(Uuid(intval))
        } else {
            Err(())
        }
    }
}

impl TryFrom<&str> for Uuid {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse(value)
    }
}

impl TryFrom<String> for Uuid {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Uuid::parse(value)
    }
}

#[allow(non_upper_case_globals)]
pub mod wellknown {
    use super::Uuid;

    pub const Nil: Uuid = Uuid::from_value(0);

    pub const NS_DNS: Uuid = Uuid::from_value(143098242404177361603877621312831893704);
    pub const NS_URL: Uuid = Uuid::from_value(143098242483405524118141958906375844040);
    pub const NS_OID: Uuid = Uuid::from_value(143098242562633686632406296499919794376);
    pub const NS_X500: Uuid = Uuid::from_value(143098242721090011660934971687007695048);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_bits() {
        let extver = move |uuid: Uuid| (uuid.0.to_be_bytes()[6] & 0xf0) >> 4;

        // Version 3
        assert_eq!(extver(crate::gen::v3(b"some_random_name", None)), 0x3);
        assert_eq!(
            extver(crate::gen::v3(b"some_random_name", Some(wellknown::NS_URL))),
            0x3
        );

        // Version 4
        assert_eq!(extver(crate::gen::v4()), 0x4);

        // Version 5
        assert_eq!(extver(crate::gen::v5(b"another_random_name", None)), 0x5);
        assert_eq!(
            extver(crate::gen::v5(b"yet_another_random_name", Some(wellknown::NS_OID))),
            0x5
        );
    }

    #[test]
    fn test_format() {
        let uuid = Uuid::from_value(339909213143343215632204095398962575718);

        assert_eq!(format!("{uuid}"), "ffb82219-2be8-4961-8c83-2163e1b4b966");
        assert_eq!(uuid.to_string_hex(), "ffb82219-2be8-4961-8c83-2163e1b4b966");
        assert_eq!(
            uuid.to_string_hex_joined(),
            "ffb822192be849618c832163e1b4b966"
        );
    }

    #[test]
    fn test_parse() {
        let uuid = crate::gen::v4();

        assert_eq!(uuid, uuid.to_string_hex().try_into().unwrap());
        assert_eq!(uuid, uuid.to_string_hex_joined().try_into().unwrap());
        assert_eq!(uuid, Uuid::from_value(uuid.value()));
    }

    #[test]
    fn test_v3_output() {
        let v3 = crate::gen::v3(b"barfoo", Some(wellknown::NS_X500));
        assert_eq!(v3.to_string_hex(), "838ae739-5539-3a99-a67b-8e291e001842");
    }

    #[test]
    fn test_v5_output() {
        let v5 = crate::gen::v5(b"foobar", Some(wellknown::NS_DNS));
        assert_eq!(v5.to_string_hex(), "a050b517-6677-5119-9a77-2d26bbf30507");
    }
}
