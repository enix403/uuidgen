use crate::uuid::{Octets, Uuid};

use digest::Digest;
use md5::Md5;
use sha1::Sha1;

fn hash_based_uuid<D: Digest>(
    mut hasher: D,
    namespace: Option<Uuid>,
    name: &[u8],
    version_hi: u8,
) -> Uuid {
    let namespace = namespace.unwrap_or_else(|| crate::uuid_v4::v4());
    hasher.update(namespace.value().to_be_bytes());
    hasher.update(name);
    let hash = hasher.finalize();

    let octets: Octets = hash[0..=15].try_into().unwrap();

    Uuid::from_octets(octets, version_hi)
}

pub fn v3(name: &[u8], namespace: Option<Uuid>) -> Uuid {
    hash_based_uuid(<Md5 as Digest>::new(), namespace, name, 0x03)
}

pub fn v5(name: &[u8], namespace: Option<Uuid>) -> Uuid {
    hash_based_uuid(<Sha1 as Digest>::new(), namespace, name, 0x05)
}


#[cfg(test)]
mod tests {
    use crate::wellknown;

    #[test]
    fn test_v3_output() {
        let v3 = super::v3(b"barfoo", Some(wellknown::NS_X500));
        assert_eq!(v3.to_string_hex(), "838ae739-5539-3a99-a67b-8e291e001842");
    }

    #[test]
    fn test_v5_output() {
        let v5 = super::v5(b"foobar", Some(wellknown::NS_DNS));
        assert_eq!(v5.to_string_hex(), "a050b517-6677-5119-9a77-2d26bbf30507");
    }
}
