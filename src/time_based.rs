#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(dead_code)]

use core::num::NonZeroU64;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::uuid::{Uuid, Octets};

pub struct TimeUuidGenerator {
    // 48-bits MAC address
    node_id: u64,

    // Unix timestamp since epoch of last generated UUID
    time_msec: Option<NonZeroU64>,

    // 14-bit counter
    clock_seq: u16,

    // Number of UUIDs generated in the same value of time_msec.
    // Resets back to 0 when time_msec changes
    generated_count: i32,
}

impl TimeUuidGenerator {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            time_msec: None,
            clock_seq: 0,
            generated_count: 0,
        }
    }

    pub fn generate(&mut self) -> Option<Uuid> {
        let last_msec = self.time_msec.map(|x| x.get()).unwrap_or(0);

        // Get the current timestamp
        let now = SystemTime::now();
        let msec = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as _;

        // TODO: if the node_id has changed, then randomize clock_seq.

        // Clock has regressed. Bump clock sequence
        if msec < last_msec {
            self.clock_seq = self.clock_seq.wrapping_add(1) & 0x3fff;
        }

        // Reset generated_count when moving to a different time interval
        if msec != last_msec {
            self.generated_count = 0;
        } else {
            self.generated_count = self.generated_count.wrapping_add(1);
        }

        // Reject if too many UUIDs are generated in a single time interval.
        if self.generated_count >= 10000 {
            return None;
        }

        // Update the last generated timestamp
        self.time_msec = NonZeroU64::new(msec);

        // Convert to 100-nanoseconds since 1582-10-15T00:00:00Z
        let ts = (msec + 12219292800000) * 10000 + self.generated_count as u64;


        // This stores the individual bytes of the timestamp with
        // the least significant byte first
        // 
        // index 0 => byte 0 => bits 00 - 07
        // index 1 => byte 1 => bits 08 - 15
        // index 2 => byte 2 => bits 16 - 23
        // index 3 => byte 3 => bits 24 - 31
        // index 4 => byte 4 => bits 32 - 39
        // index 5 => byte 5 => bits 40 - 47
        // index 6 => byte 6 => bits 48 - 55
        // index 7 => byte 7 => bits 56 - 63
        let ts_bytes = ts.to_le_bytes();

        let mut octets = Octets::default();

        // Set the time_low field equal to the least significant 32
        // bits of the timestamp
        octets[0..=3].copy_from_slice(&ts_bytes[0..=3]);

        // Set the time_mid field equal to bits 32 through 47 of the timestamp
        octets[4..=5].copy_from_slice(&ts_bytes[4..=5]);

        // Set the 12 least significant bits (bits 0 through 11) of the
        // time_hi_and_version field equal to bits 48 through 59 from the
        // timestamp. The most significant bits are ignored (actually overwritten,
        // see below) after copying
        octets[6..=7].copy_from_slice(&ts_bytes[6..=7]);

        // Set the clock_seq_low field to the eight least significant bits
        // (bits zero through 7) of the clock sequence.
        octets[9] = (self.clock_seq & 0x00ff) as _;

        // Set the 6 least significant bits (bits zero through 5) of the
        // clock_seq_hi_and_reserved field to the 6 most significant bits
        // (bits 8 through 13) of the clock sequence.
        octets[8] = ((self.clock_seq & 0xff00) >> 8) as _;

        // Set the node field to the 48-bit IEEE address in the same order of
        // significance as the address.
        octets[10..=15].copy_from_slice(self.node_id.to_ne_bytes().as_slice());

        Some(Uuid::from_octets(octets, 0x10))
    }
}
