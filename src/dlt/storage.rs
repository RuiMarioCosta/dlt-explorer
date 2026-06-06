/// DLT storage header delimiter.
pub const STORAGE_HEADER_PATTERN: &[u8] = b"DLT\x01";

// Storage header sizes (matches DltStorageHeader in dlt-daemon)
pub const STORAGE_HEADER_PATTERN_SIZE: usize = 4;
pub const STORAGE_HEADER_SECONDS_SIZE: usize = 4;
pub const STORAGE_HEADER_MICROSECONDS_SIZE: usize = 4;
pub const STORAGE_HEADER_ECU_SIZE: usize = 4;

/// Storage header total size in bytes.
pub const STORAGE_HEADER_SIZE: usize = STORAGE_HEADER_PATTERN_SIZE
    + STORAGE_HEADER_SECONDS_SIZE
    + STORAGE_HEADER_MICROSECONDS_SIZE
    + STORAGE_HEADER_ECU_SIZE;

