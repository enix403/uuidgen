#![allow(dead_code)]
#![allow(unused_variables)]

use crate::uuid::Uuid;

#[derive(Debug)]
pub struct UuidFields {
    /// Number of milliseconds since the *Unix epoch* (January 1st, 1970)
    pub time: u64,

    /// The 4-bit version field of the UUID
    pub version: u8,

    /// The 3-bit variant field of the UUID
    pub variant: u8,

    /// Clock sequence of the UUID
    pub clock_seq: u16,

    /// The 48-bit node field of the UUID
    pub node_id: u64
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
            let mut bytes = [0; 8];
            bytes[2..8].copy_from_slice(&octets[10..=15]);
            bytes
        });

        /* ========================= */
        /* Parse/Assemble the fields */
        /* ========================= */

        let version = ((time_hi_and_version & 0xf000) >> 12) as u8;

        let time_epoch_millisecs = {
            // Remove the version bits from timestamp
            let time_hi_and_version = (time_hi_and_version & 0x0fff) as u64;
            let time_mid = time_mid as u64;
            let time_low = time_low as u64;

            // Combine the 3 fields into full timestamp. This will represent the
            // count of 100-nanosecond intervals since 00:00:00.00, 15 October 1582
            let uuidtime = time_hi_and_version << 48 | time_mid << 32 | time_low;

            // Convert to milliseconds
            let time_milli = uuidtime / 10000;

            // Convert to milliseconds since Unix Epoch (00:00:00.00, 1 January 1970)
            time_milli.wrapping_sub(12219292800000)
        };

        // The clk_seq_hi_res field contains both the variant the high byte of clock
        // sequence, but the two fields are encoded as variable number of bits, as shown
        // in the table below.
        //
        // Bits labelled 0 or 1 contitute the variant part, while the remaining bits (Y)
        // make up the clock sequence.
        //
        // Bit 7 6 5 4 3 2 1 0    Variant
        //     0 Y Y Y Y Y Y Y => Reserved, NCS backward compatibility.
        //     1 0 Y Y Y Y Y Y => The variant specified in RFC 4122.
        //     1 1 0 Y Y Y Y Y => Reserved, Microsoft Corporation backward compatibility
        //     1 1 1 Y Y Y Y Y => Reserved for future definition.
        let vmask = match clk_seq_hi_res {
            // Check if bit 7 is 0
            // In this case only bit 7 constitutes variant
            x if x < 0b_1000_0000 => 0x80,

            // Otherwise check if bit 6 is 0
            // In this case bit 7 and 6 constitute variant
            x if x < 0b_1100_0000 => 0xC0,

            // Otherwise bit 7, 6 and 5 constitute variant
            _ => 0xE0
        };

        let variant = clk_seq_hi_res & vmask;
        let clk_seq_hi = clk_seq_hi_res & !vmask;

        let clock_seq = ((clk_seq_hi as u16) << 8) | clk_seq_low as u16;

        Self {
            time: time_epoch_millisecs,
            version,
            variant,
            clock_seq,
            node_id,
        }
    }
}
