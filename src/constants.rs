/// Number of milliseconds between 00:00:00.00, 15 October 1582 and
/// 00:00:00.00, 01 January 1970
pub const MILLISECS_GREGORIAN_UNIX: u64 = 12219292800000;

/// Number of nanoseconds between 00:00:00.00, 15 October 1582 and
/// 00:00:00.00, 01 January 1970
pub const NANOSECS_GREGORIAN_UNIX: u64 = MILLISECS_GREGORIAN_UNIX * 1_000_000;

