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
    pub const fn from_val(value: u128) -> Self {
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
        namespace: Option<UUID>,
        name: &[u8],
        version: u8,
    ) -> Self {
        let namespace = namespace.unwrap_or_else(|| Self::v4());
        hasher.update(namespace.0.to_be_bytes());
        hasher.update(name);

        let hash = hasher.finalize();
        let hash = &hash[..];

        let mut octets = [0u8; 16];

        octets.copy_from_slice(&hash[0..=15]);
        Self::finalize_octets(octets, version)
    }

    pub fn v3(name: &[u8], namespace: Option<UUID>) -> Self {
        Self::hash_based_uuid(<Md5 as Digest>::new(), namespace, name, 3)
    }

    pub fn v4() -> UUID {
        let mut octets = [0u8; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut octets);

        Self::finalize_octets(octets, 4)
    }

    pub fn v5(name: &[u8], namespace: Option<UUID>) -> Self {
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

#[allow(non_upper_case_globals)]
pub mod wellknown {
    use super::UUID;

    pub const Nil: UUID = UUID::from_val(0);

    pub const NS_DNS: UUID = UUID::from_val(143098242404177361603877621312831893704);
    pub const NS_URL: UUID = UUID::from_val(143098242483405524118141958906375844040);
    pub const NS_OID: UUID = UUID::from_val(143098242562633686632406296499919794376);
    pub const NS_X500: UUID = UUID::from_val(143098242721090011660934971687007695048);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_bits() {
        let extver = move |uuid: UUID| (uuid.0.to_be_bytes()[6] & 0xf0) >> 4;

        // Version 3
        assert_eq!(extver(UUID::v3(b"some_random_name", None)), 0x3);
        assert_eq!(
            extver(UUID::v3(b"some_random_name", Some(wellknown::NS_URL))),
            0x3
        );

        // Version 4
        assert_eq!(extver(UUID::v4()), 0x4);

        // Version 5
        assert_eq!(extver(UUID::v5(b"another_random_name", None)), 0x5);
        assert_eq!(
            extver(UUID::v5(b"yet_another_random_name", Some(wellknown::NS_OID))),
            0x5
        );
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
