use core::cell::RefCell;

use rand::RngCore;

use crate::uuid::{Octets, Uuid};

pub trait NodeIdProvider {
    fn get_node_id(&self) -> u64;
}

#[derive(Clone, Copy)]
pub struct RandomNodeIdProvider;

impl NodeIdProvider for RandomNodeIdProvider {
    fn get_node_id(&self) -> u64 {
        // Per Section 4.5:
        // For a randomly generated node id, the least significant bit of the first octet
        // must be set to one
        rand::thread_rng().next_u64() | 0x01_00_00_00_00_00_00_00
    }
}

#[derive(Clone)]
pub struct StaticNodeIdProvider(u64);

impl StaticNodeIdProvider {
    pub fn new(node_id: u64) -> Self {
        Self(node_id)
    }
}

impl NodeIdProvider for StaticNodeIdProvider {
    fn get_node_id(&self) -> u64 {
        self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Too many UUIDs generated in a single time interval")]
    TooManyGenerated,
}

pub struct TimeBasedState {
    // 48-bits MAC address
    node_id: u64,

    // Unix timestamp in milliseconds of last generated UUID
    time_msec: u64,

    // Clock Sequence: a 14-bit counter as per RFC
    clock_seq: u16,

    // Number of UUIDs generated in the same value of time_msec.
    // Resets back to 0 when time_msec changes
    generated_count: i32,
}

struct TimeBasedGenerator<const V: u8, P> {
    node_id_provider: P,
    state: TimeBasedState,
}

impl<const V: u8, P> TimeBasedGenerator<V, P>
where
    P: NodeIdProvider,
{
    fn get_time_milli() -> u64 {
        #[cfg(feature = "webtime")]
        use web_time::{SystemTime, UNIX_EPOCH};

        #[cfg(not(feature = "webtime"))]
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn new(node_id_provider: P) -> Self {
        let node_id = node_id_provider.get_node_id();
        Self {
            node_id_provider,
            state: TimeBasedState {
                node_id,
                time_msec: Self::get_time_milli(),
                clock_seq: (rand::thread_rng().next_u32() & 0x0000ffff) as u16,
                generated_count: 0,
            },
        }
    }

    fn new_with_state(node_id_provider: P, state: TimeBasedState) -> Self {
        Self {
            node_id_provider,
            state,
        }
    }

    fn generate(&mut self) -> Result<Uuid, Error> {
        // Get the current timestamp
        let msec = Self::get_time_milli();

        let octets = Self::layout_octets(&self.state);

        let next_state = Self::tick(&self.state, self.node_id_provider.get_node_id(), msec)?;
        self.state = next_state;

        Ok(Uuid::from_octets(octets, V))
    }

    fn layout_octets(state: &TimeBasedState) -> Octets {
        // Convert to 100-nanoseconds since 1582-10-15T00:00:00Z
        let ts = (state.time_msec + crate::constants::MILLISECS_GREGORIAN_UNIX) * 10000
            + state.generated_count as u64;

        // This stores the individual bytes of the timestamp with
        // the most significant byte first
        //
        // index 0 => byte 7 => bits 56 - 63
        // index 1 => byte 6 => bits 48 - 55
        //
        // index 2 => byte 5 => bits 40 - 47
        // index 3 => byte 4 => bits 32 - 39
        //
        // index 4 => byte 3 => bits 24 - 31
        // index 5 => byte 2 => bits 16 - 23
        // index 6 => byte 1 => bits 08 - 15
        // index 7 => byte 0 => bits 00 - 07
        let ts_bytes = ts.to_be_bytes();

        let mut octets = Octets::default();

        // Set the time_low field equal to the least significant 32
        // bits of the timestamp
        octets[0..=3].copy_from_slice(&ts_bytes[4..=7]);

        // Set the time_mid field equal to bits 32 through 47 of the timestamp
        octets[4..=5].copy_from_slice(&ts_bytes[2..=3]);

        // Set the 12 least significant bits (bits 0 through 11) of the
        // time_hi_and_version field equal to bits 48 through 59 from the
        // timestamp. The remaining bits are overwritten later.
        octets[6..=7].copy_from_slice(&ts_bytes[0..=1]);

        // Set the clock_seq_low field to the eight least significant bits
        // (bits zero through 7) of the clock sequence.
        octets[9] = (state.clock_seq & 0x00ff) as _;

        // Set the 6 least significant bits (bits zero through 5) of the
        // clock_seq_hi_and_reserved field to the 6 most significant bits
        // (bits 8 through 13) of the clock sequence. The remaining bits
        // are overwritten later.
        octets[8] = ((state.clock_seq & 0x3f00) >> 8) as _;

        // Set the node field to the 48-bit IEEE address in the same order of
        // significance as the address.
        octets[10..=15].copy_from_slice(&state.node_id.to_be_bytes()[2..8]);

        octets
    }

    fn tick(state: &TimeBasedState, node_id: u64, msec: u64) -> Result<TimeBasedState, Error> {
        let last_msec = state.time_msec;
        let mut clock_seq = state.clock_seq;
        let mut generated_count = state.generated_count;

        // If the node_id has changed, then reset clock_seq with a random value.
        if state.node_id != node_id {
            clock_seq = (rand::thread_rng().next_u32() & 0x0000ffff) as u16;
        }

        // Clock has regressed. Bump clock sequence
        if msec < last_msec {
            clock_seq = clock_seq.wrapping_add(1) & 0x3fff;
        }

        // Reset generated_count when moving to a different time interval
        if msec != last_msec {
            generated_count = 0;
        } else {
            generated_count = generated_count.wrapping_add(1);
        }

        // Reject if too many UUIDs are generated in a single time interval.
        if generated_count >= 10000 {
            return Err(Error::TooManyGenerated);
        }

        Ok(TimeBasedState {
            node_id,
            time_msec: msec,
            clock_seq,
            generated_count,
        })
    }
}

#[repr(transparent)]
pub struct V1Generator<P>(TimeBasedGenerator<1, P>);

impl<P> V1Generator<P>
where
    P: NodeIdProvider,
{
    #[inline(always)]
    pub fn new(node_id_provider: P) -> Self {
        Self(TimeBasedGenerator::new(node_id_provider))
    }

    #[inline(always)]
    pub fn generate(&mut self) -> Result<Uuid, Error> {
        self.0.generate()
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn new_with_state(node_id_provider: P, state: TimeBasedState) -> Self {
        Self(
            //
            TimeBasedGenerator::new_with_state(node_id_provider, state),
        )
    }
}

thread_local! {
    static GLOBAL_GENERATOR_V1: RefCell<V1Generator<RandomNodeIdProvider>> = RefCell::new(
        V1Generator::new(
            RandomNodeIdProvider
        )
    );
}

pub fn v1() -> Result<Uuid, Error> {
    GLOBAL_GENERATOR_V1.with(|generator| generator.borrow_mut().generate())
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_output() {
        // *Reference UUID to test with*
        //
        // Standard String Format  d71c7cd2-aa3b-11ee-ac4a-325096b39f47
        // Single Integer Value    285931935115184731626516849736750964551
        // Version                 1 (time and node based)
        // Variant                 DCE 1.1, ISO/IEC 11578:1996
        // Contents - Time         2024-01-03 13:27:28.382485.0 UTC
        // Contents - Clock        11338
        // Contents - Node         32:50:96:b3:9f:47 (local unicast)

        let node_id: u64 = 0x_32_50_96_B3_9F_47;
        let clock_seq: u16 = 11338;

        let time_msec = 1704288448382; // Time in milliseconds resolution
        let num_100_nanosecs = 4850;

        let node_id_provider = StaticNodeIdProvider(node_id);

        let state = TimeBasedState {
            node_id,
            time_msec,
            clock_seq,
            generated_count: num_100_nanosecs,
        };

        let mut generator = V1Generator::new_with_state(node_id_provider, state);

        let value = generator.generate().unwrap();
        assert_eq!(
            value.to_string_hex(),
            "d71c7cd2-aa3b-11ee-ac4a-325096b39f47"
        );
    }
}
