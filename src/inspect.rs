#![allow(dead_code)]
#![allow(unused_variables)]

use crate::uuid::Uuid;

#[derive(Debug)]
pub struct UuidFields {
    /// Number of milliseconds since the *Unix epoch* (January 1st, 1970)
    time: u64,

    /// The 4-bit version field of the UUID
    version: u8,

    /// The 3-bit variant field of the UUID
    variant: u8,

    /// Clock sequence of the UUID
    clock_seq: u16,

    /// The 48-bit node field of the UUID
    node_id: u64
}

impl UuidFields {
    pub fn new(uuid: Uuid) -> Self {
        let octets = uuid.value().to_be_bytes();

        // Fields of the UUID as per RFC section 4.1.2
        let time_low = u32::from_be_bytes(octets[00..=03].try_into().unwrap());
        let time_mid = ((octets[4] as u16) << 8) | octets[5] as u16;
        let time_hi_and_version = ((octets[6] as u16) << 8) | octets[7] as u16;
        let clk_seq_hi_res = octets[8];
        let clk_seq_low = octets[9];

        let node_id = u64::from_be_bytes({
            let mut bytes = [0u8; 8];
            bytes[2..8].copy_from_slice(&octets[10..=15]);
            bytes
        });

        let version = ((time_hi_and_version & 0xf000) >> 12) as u8;
        let variant = ((clk_seq_hi_res & 0xe0) >> 5) as u8;


        let clock_seq = ((clk_seq_hi_res as u16) << 8) | clk_seq_low as u16;
        let clock_seq = clock_seq & 0x1fff;

        let time_epoch_millisecs = {
            // remove the version bits from timestamp
            let time_hi_and_version = (time_hi_and_version & 0x0fff) as u64;
            let time_mid = time_mid as u64;
            let time_low = time_low as u64;

            // Combine the 3 fields into full timestamp
            let uuidtime = time_hi_and_version << 48 | time_mid << 32 | time_low;

            // Convert to milliseconds
            let time_milli = uuidtime / 10000;

            // Convert milliseconds
            time_milli - 12219292800000
        };

        // println!("time_epoch = {}", time_epoch_millisecs);
        // println!("version = {}", version);
        // println!("variant = {:0b}", variant);
        // println!("clock_seq = {}", clock_seq);
        // println!("node_id = {:#018x}", node_id);

        Self {
            time: time_epoch_millisecs,
            version,
            variant,
            clock_seq,
            node_id,
        }
    }
}
