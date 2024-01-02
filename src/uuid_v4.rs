use crate::uuid::{Octets, Uuid};

use rand::RngCore;

pub fn v4() -> Uuid {
    let mut octets = Octets::default();
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut octets);

    Uuid::from_octets(octets, 0x4)
}
