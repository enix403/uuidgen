#![allow(dead_code)]
#![allow(unused_variables)]

use num_traits::Num;
use num_traits::cast::FromPrimitive;
use num_traits::cast::ToPrimitive;

use crate::uuid::Uuid;

/// The individual fields of a UUID as per RFC 4122. Each field is stored in
/// Big-Endian order
#[derive(Clone, Debug)]
pub struct UuidFields {
    time_low: u32,
    time_mid: u16,
    time_hi_and_version: u16,
    clk_seq_hi_res: u8,
    clk_seq_low: u8,
    node: u64,
}

impl UuidFields {
    pub fn of(uuid: &Uuid) -> Self {
        let octets = uuid.value().to_be_bytes();

        // Fields of the UUID as per RFC section 4.1.2
        let time_low = u32::from_be_bytes(octets[00..=03].try_into().unwrap());
        let time_mid = ((octets[4] as u16) << 8) | octets[5] as u16;
        let time_hi_and_version = ((octets[6] as u16) << 8) | octets[7] as u16;
        let clk_seq_hi_res = octets[8];
        let clk_seq_low = octets[9];

        let node = u64::from_be_bytes({
            let mut bytes = [0; 8];
            bytes[2..8].copy_from_slice(&octets[10..=15]);
            bytes
        });

        Self {
            time_low,
            time_mid,
            time_hi_and_version,
            clk_seq_hi_res,
            clk_seq_low,
            node,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UuidDetails {
    /// The 60-bit field of the UUID. This is the number of 100-nanosecond intervals
    /// since 00:00:00.00, 15 October 1582
    pub time: u64,

    /// The 4-bit version field of the UUID
    pub version: u8,

    /// The 8-bit variant field of the UUID.
    ///
    /// UUID variant is encoded in a variable number of bits. For this reason this contains
    /// the full octet containing the variant, but with all the clock sequence bits set to 0
    pub variant: u8,

    /// The 14-bit clock sequence of the UUID
    pub clock_seq: u16,

    /// The 48-bit node field of the UUID
    pub node: u64,
}

/// The time information returned by `UuidFields::unix_time()` method
///
/// UUIDs contain a timestamp with resolution upto nanoseconds, which sometimes doesn't fit
/// in a single 64-bit integer. This struct contains the timestamp in a destructed way in multiple
/// integers, namely `seconds`, `microseconds` and `hecto_nanoseconds`. `seconds` stores the number of
/// seconds in the timestamp. `microseconds` stores the additional microseconds, after one has taken
/// `seconds` into the account. Similarly, `hecto_nanoseconds` contains the additional 100-nanoseconds
/// intervals, after one taken into account `microseconds`.
///
/// 'Hecto' is a standard SI prefix meaning 100 
#[derive(Clone, Debug)]
pub struct TimeSpec {
    /// Seconds of the timestamp
    pub seconds: u64,

    /// Microseconds of the timestamp
    pub microseconds: i32,

    /// 100-nanoseconds of the timestamp.
    pub hecto_nanoseconds: i8,
}

impl TimeSpec {
    fn zero() -> Self {
        Self {
            seconds: 0,
            microseconds: 0,
            hecto_nanoseconds: 0
        }
    }
}

impl UuidDetails {
    pub fn construct(fields: &UuidFields) -> Self {

        let version = ((fields.time_hi_and_version & 0xf000) >> 12) as u8;

        let time_epoch_millisecs = {
            // Remove the version bits from timestamp
            let time_hi_and_version = (fields.time_hi_and_version & 0x0fff) as u64;
            let time_mid = fields.time_mid as u64;
            let time_low = fields.time_low as u64;

            // Combine the 3 fields into full timestamp. This will represent the
            // count of 100-nanosecond intervals since 00:00:00.00, 15 October 1582
            let uuidtime = time_hi_and_version << 48 | time_mid << 32 | time_low;

            uuidtime
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
        let clk_seq_hi_res = fields.clk_seq_hi_res;
        let vmask = match clk_seq_hi_res {
            // Check if bit 7 is 0
            // In this case only bit 7 constitutes variant
            x if x < 0b_1000_0000 => 0x80,

            // Otherwise check if bit 6 is 0
            // In this case bit 7 and 6 constitute variant
            x if x < 0b_1100_0000 => 0xC0,

            // Otherwise bit 7, 6 and 5 constitute variant
            _ => 0xE0,
        };

        let variant = clk_seq_hi_res & vmask;
        let clk_seq_hi = clk_seq_hi_res & !vmask;

        let clock_seq = ((clk_seq_hi as u16) << 8) | fields.clk_seq_low as u16;

        Self {
            time: time_epoch_millisecs,
            version,
            variant,
            clock_seq,
            node: fields.node,
        }
    }

    /// Returns the timestamp of the UUID. See [`TimeSpec`] for more details about
    /// the return type 
    pub fn unix_time(&self) -> TimeSpec {
        // The offset in 100-nanosecond intervals
        let offset = crate::constants::MILLISECS_GREGORIAN_UNIX * 10000;

        let mut time = ConsumingU64(self.time.saturating_sub(offset));

        let mut timespec = TimeSpec::zero();

        timespec.hecto_nanoseconds = time.divn_mod(10);
        timespec.microseconds = time.divn_mod(1000000);
        timespec.seconds = time.remaining();

        timespec
    }
}

struct ConsumingU64(u64);

impl ConsumingU64 {
    fn divn_mod<T>(&mut self, n: T) -> T
    where
        T: Num + ToPrimitive + FromPrimitive,
    {
        let n = n.to_u64().unwrap();
        let rem = self.0 % n;
        self.0 /= n;
        T::from_u64(rem).unwrap()
    }

    fn remaining(self) -> u64 {
        self.0
    }
}
