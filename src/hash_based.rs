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
