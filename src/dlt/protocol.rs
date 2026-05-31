// AUTOSAR PRS v2 protocol constants and bitfield helpers

/// DLT storage header delimiter
pub const DLT_STORAGE_HEADER_PATTERN: &[u8] = b"DLT\x01";

// Storage header sizes (matches DltStorageHeader in dlt-daemon)
pub const STORAGE_HEADER_PATTERN_SIZE: usize = 4;
pub const STORAGE_HEADER_SECONDS_SIZE: usize = 4;
pub const STORAGE_HEADER_MICROSECONDS_SIZE: usize = 4;
pub const STORAGE_HEADER_ECU_SIZE: usize = 4;
pub const STORAGE_HEADER_SIZE: usize =
    STORAGE_HEADER_PATTERN_SIZE + STORAGE_HEADER_SECONDS_SIZE + STORAGE_HEADER_MICROSECONDS_SIZE + STORAGE_HEADER_ECU_SIZE; // 16

/// Base header minimum size: HTYP2(4) + MCNT(1) + LEN(2) = 7
pub const BASE_HEADER_MIN_SIZE: usize = 7;

// ---------------------------------------------------------------------------
// v1 HTYP bit definitions
// ---------------------------------------------------------------------------

/// MSBF bit in v1 HTYP — indicates big-endian payload byte order.
pub const DLT_HTYP_MSBF: u8 = 0x02;

/// Returns true if the v1 HTYP byte has the MSBF (most significant byte first) flag set.
#[inline]
pub fn htyp_has_msbf(htyp: u8) -> bool {
    htyp & DLT_HTYP_MSBF != 0
}

// ---------------------------------------------------------------------------
// HTYP2 bitfield layout
//
// HTYP2 is a 32-bit big-endian field.  Byte 0 (transmitted first) holds:
//   bits 0-1  CNTI   bits 24-25 of the u32
//   bit  2    WEID   bit 26
//   bit  3    WACID  bit 27
//   bit  4    WSID   bit 28
//   bits 5-7  VERS   bits 29-31
// Byte 1 holds:
//   bit  8    WSFLN  bit 16
//   bit  9    WTGS   bit 17
//   bit 10    WPVL   bit 18
//   bit 11    WSGM   bit 19
// ---------------------------------------------------------------------------

// CNTI (Content Info) — bits 0-1 of byte 0
pub const HTYP2_CNTI_SHIFT: u32 = 24;
pub const HTYP2_CNTI_MASK: u32 = 0x03 << HTYP2_CNTI_SHIFT;
pub const CNTI_VERBOSE: u8 = 0x00;
pub const CNTI_NON_VERBOSE: u8 = 0x01;
pub const CNTI_CONTROL: u8 = 0x02;

// WEID — bit 2 of byte 0
pub const HTYP2_WEID: u32 = 1 << 26;

// WACID — bit 3 of byte 0
pub const HTYP2_WACID: u32 = 1 << 27;

// WSID — bit 4 of byte 0
pub const HTYP2_WSID: u32 = 1 << 28;

// VERS — bits 5-7 of byte 0
pub const HTYP2_VERS_SHIFT: u32 = 29;
pub const HTYP2_VERS_MASK: u32 = 0x07 << HTYP2_VERS_SHIFT;
pub const PROTOCOL_VERSION_2: u8 = 2;

// Byte 1 flags
pub const HTYP2_WSFLN: u32 = 1 << 16;
pub const HTYP2_WTGS: u32 = 1 << 17;
pub const HTYP2_WPVL: u32 = 1 << 18;
pub const HTYP2_WSGM: u32 = 1 << 19;

// ---------------------------------------------------------------------------
// MSIN byte layout (present for verbose and control messages)
//   bit  0     reserved
//   bits 1-3   MSTP  (Message Type)
//   bits 4-7   MTIN  (Message Type Info — meaning depends on MSTP)
// ---------------------------------------------------------------------------

// MSTP values (Message Type)
pub const MESSAGE_TYPE_LOG: u8 = 0x00;
pub const MESSAGE_TYPE_TRACE: u8 = 0x01;
pub const MESSAGE_TYPE_NETWORK: u8 = 0x02;
pub const MESSAGE_TYPE_CONTROL: u8 = 0x03;

// MTIN values when MSTP = LOG (Log Level)
pub const LOG_LEVEL_FATAL: u8 = 0x01;
pub const LOG_LEVEL_ERROR: u8 = 0x02;
pub const LOG_LEVEL_WARN: u8 = 0x03;
pub const LOG_LEVEL_INFO: u8 = 0x04;
pub const LOG_LEVEL_DEBUG: u8 = 0x05;
pub const LOG_LEVEL_VERBOSE: u8 = 0x06;

#[inline]
pub fn htyp2_cnti(htyp2: u32) -> u8 {
    ((htyp2 & HTYP2_CNTI_MASK) >> HTYP2_CNTI_SHIFT) as u8
}

#[inline]
pub fn htyp2_version(htyp2: u32) -> u8 {
    ((htyp2 & HTYP2_VERS_MASK) >> HTYP2_VERS_SHIFT) as u8
}

#[inline]
pub fn htyp2_has_weid(htyp2: u32) -> bool {
    htyp2 & HTYP2_WEID != 0
}

#[inline]
pub fn htyp2_has_wacid(htyp2: u32) -> bool {
    htyp2 & HTYP2_WACID != 0
}

#[inline]
pub fn htyp2_has_wsid(htyp2: u32) -> bool {
    htyp2 & HTYP2_WSID != 0
}

#[inline]
pub fn htyp2_has_wsfln(htyp2: u32) -> bool {
    htyp2 & HTYP2_WSFLN != 0
}

#[inline]
pub fn htyp2_has_wtgs(htyp2: u32) -> bool {
    htyp2 & HTYP2_WTGS != 0
}

#[inline]
pub fn htyp2_has_wpvl(htyp2: u32) -> bool {
    htyp2 & HTYP2_WPVL != 0
}

#[inline]
pub fn htyp2_has_wsgm(htyp2: u32) -> bool {
    htyp2 & HTYP2_WSGM != 0
}

// MSIN helpers
#[inline]
pub fn msin_mstp(msin: u8) -> u8 {
    (msin >> 1) & 0x07
}

#[inline]
pub fn msin_mtin(msin: u8) -> u8 {
    (msin >> 4) & 0x0F
}

/// Build an MSIN byte from MSTP and MTIN fields.
#[inline]
pub fn build_msin(mstp: u8, mtin: u8) -> u8 {
    ((mstp & 0x07) << 1) | ((mtin & 0x0F) << 4)
}

/// Build a HTYP2 u32 value from individual fields.
pub fn build_htyp2(cnti: u8, weid: bool, wacid: bool, wsid: bool, vers: u8) -> u32 {
    let mut byte0: u8 = cnti & 0x03;
    if weid {
        byte0 |= 1 << 2;
    }
    if wacid {
        byte0 |= 1 << 3;
    }
    if wsid {
        byte0 |= 1 << 4;
    }
    byte0 |= (vers & 0x07) << 5;
    (byte0 as u32) << 24
}

/// Build a HTYP2 u32 value with byte-1 flags included.
pub fn build_htyp2_full(
    cnti: u8,
    weid: bool,
    wacid: bool,
    wsid: bool,
    vers: u8,
    wsfln: bool,
    wtgs: bool,
    wpvl: bool,
    wsgm: bool,
) -> u32 {
    let mut val = build_htyp2(cnti, weid, wacid, wsid, vers);
    if wsfln {
        val |= HTYP2_WSFLN;
    }
    if wtgs {
        val |= HTYP2_WTGS;
    }
    if wpvl {
        val |= HTYP2_WPVL;
    }
    if wsgm {
        val |= HTYP2_WSGM;
    }
    val
}

/// Encode a nanosecond timestamp into a 9-byte base header TMSP2 field (big-endian).
/// This is the in-message timestamp per AUTOSAR PRS, NOT the storage header timestamp.
pub fn encode_tmsp2(total_ns: u64) -> [u8; 9] {
    let seconds = total_ns / 1_000_000_000;
    let nanoseconds = (total_ns % 1_000_000_000) as u32;
    let mut buf = [0u8; 9];
    buf[0..4].copy_from_slice(&nanoseconds.to_be_bytes());
    buf[4] = (seconds >> 32) as u8;
    buf[5] = (seconds >> 24) as u8;
    buf[6] = (seconds >> 16) as u8;
    buf[7] = (seconds >> 8) as u8;
    buf[8] = seconds as u8;
    buf
}

/// Decode a 9-byte base header TMSP2 field into a nanosecond timestamp.
/// This is the in-message timestamp per AUTOSAR PRS, NOT the storage header timestamp.
pub fn decode_tmsp2(tmsp2: &[u8; 9]) -> u64 {
    let nanoseconds = u32::from_be_bytes(tmsp2[0..4].try_into().unwrap()) & 0x7FFF_FFFF;
    let seconds = ((tmsp2[4] as u64) << 32)
        | ((tmsp2[5] as u64) << 24)
        | ((tmsp2[6] as u64) << 16)
        | ((tmsp2[7] as u64) << 8)
        | (tmsp2[8] as u64);
    seconds.saturating_mul(1_000_000_000).saturating_add(nanoseconds as u64)
}

// ---------------------------------------------------------------------------
// v1 HTYP bitfield definitions
// ---------------------------------------------------------------------------

const DLT_HTYP_UEH: u8 = 0x01; // use extended header
const DLT_HTYP_WEID: u8 = 0x04; // with ECU ID
const DLT_HTYP_WSID: u8 = 0x08; // with session ID
const DLT_HTYP_WTMS: u8 = 0x10; // with timestamp

#[inline]
pub fn htyp_has_ueh(htyp: u8) -> bool {
    htyp & DLT_HTYP_UEH != 0
}

#[inline]
pub fn htyp_has_weid(htyp: u8) -> bool {
    htyp & DLT_HTYP_WEID != 0
}

#[inline]
pub fn htyp_has_wsid(htyp: u8) -> bool {
    htyp & DLT_HTYP_WSID != 0
}

#[inline]
pub fn htyp_has_wtms(htyp: u8) -> bool {
    htyp & DLT_HTYP_WTMS != 0
}

// ---------------------------------------------------------------------------
// v1 MSIN verb helper
// ---------------------------------------------------------------------------

#[inline]
pub fn msin_is_verb(msin: u8) -> bool {
    (msin & 0x01) != 0
}

// ---------------------------------------------------------------------------
// v1 message type / control type constants
// ---------------------------------------------------------------------------

pub const DLT_TYPE_CONTROL: u8 = 0x03;
pub const DLT_CONTROL_RESPONSE: u8 = 0x02;

// ---------------------------------------------------------------------------
// Size constants (shared)
// ---------------------------------------------------------------------------

pub const DLT_ID_SIZE: usize = 4;
pub const DLT_SIZE_WEID: usize = DLT_ID_SIZE;
pub const DLT_SIZE_WSID: usize = 4;
pub const DLT_SIZE_WTMS: usize = 4;

// ---------------------------------------------------------------------------
// v1 message classification helpers
// ---------------------------------------------------------------------------

pub fn standard_header_extra_size(htyp: u8) -> usize {
    let mut size = 0;
    if htyp_has_weid(htyp) {
        size += DLT_SIZE_WEID;
    }
    if htyp_has_wsid(htyp) {
        size += DLT_SIZE_WSID;
    }
    if htyp_has_wtms(htyp) {
        size += DLT_SIZE_WTMS;
    }
    size
}

pub fn msg_is_control(htyp: u8, msin: u8) -> bool {
    htyp_has_ueh(htyp) && (msin_mstp(msin) == DLT_TYPE_CONTROL)
}

pub fn msg_is_control_response(htyp: u8, msin: u8) -> bool {
    htyp_has_ueh(htyp)
        && (msin_mstp(msin) == DLT_TYPE_CONTROL)
        && (msin_mtin(msin) == DLT_CONTROL_RESPONSE)
}

pub fn msg_is_nonverbose(htyp: u8, msin: u8) -> bool {
    !htyp_has_ueh(htyp) || (htyp_has_ueh(htyp) && !msin_is_verb(msin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_extraction() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, true, false, PROTOCOL_VERSION_2);
        assert_eq!(htyp2_version(htyp2), 2);
    }

    #[test]
    fn cnti_extraction() {
        assert_eq!(
            htyp2_cnti(build_htyp2(CNTI_VERBOSE, false, false, false, 2)),
            CNTI_VERBOSE
        );
        assert_eq!(
            htyp2_cnti(build_htyp2(CNTI_NON_VERBOSE, false, false, false, 2)),
            CNTI_NON_VERBOSE
        );
        assert_eq!(
            htyp2_cnti(build_htyp2(CNTI_CONTROL, false, false, false, 2)),
            CNTI_CONTROL
        );
    }

    #[test]
    fn flag_extraction() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, true, true, true, 2);
        assert!(htyp2_has_weid(htyp2));
        assert!(htyp2_has_wacid(htyp2));
        assert!(htyp2_has_wsid(htyp2));
    }

    #[test]
    fn flags_cleared() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, 2);
        assert!(!htyp2_has_weid(htyp2));
        assert!(!htyp2_has_wacid(htyp2));
        assert!(!htyp2_has_wsid(htyp2));
    }

    #[test]
    fn build_roundtrip() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, true, false, PROTOCOL_VERSION_2);
        // byte0 = (2 << 5) | (1 << 3) = 0x48
        assert_eq!(htyp2, 0x48_00_00_00);
        assert_eq!(htyp2_cnti(htyp2), CNTI_VERBOSE);
        assert!(htyp2_has_wacid(htyp2));
        assert!(!htyp2_has_weid(htyp2));
        assert_eq!(htyp2_version(htyp2), PROTOCOL_VERSION_2);
    }

    #[test]
    fn byte1_flag_extraction() {
        let htyp2 = build_htyp2_full(
            CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2,
            true, true, true, true,
        );
        assert!(htyp2_has_wsfln(htyp2));
        assert!(htyp2_has_wtgs(htyp2));
        assert!(htyp2_has_wpvl(htyp2));
        assert!(htyp2_has_wsgm(htyp2));
    }

    #[test]
    fn byte1_flags_cleared() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
        assert!(!htyp2_has_wsfln(htyp2));
        assert!(!htyp2_has_wtgs(htyp2));
        assert!(!htyp2_has_wpvl(htyp2));
        assert!(!htyp2_has_wsgm(htyp2));
    }

    #[test]
    fn msin_roundtrip() {
        let msin = build_msin(MESSAGE_TYPE_LOG, LOG_LEVEL_INFO);
        assert_eq!(msin_mstp(msin), MESSAGE_TYPE_LOG);
        assert_eq!(msin_mtin(msin), LOG_LEVEL_INFO);
    }

    #[test]
    fn msin_all_types() {
        for mstp in [MESSAGE_TYPE_LOG, MESSAGE_TYPE_TRACE, MESSAGE_TYPE_NETWORK, MESSAGE_TYPE_CONTROL] {
            let msin = build_msin(mstp, 0);
            assert_eq!(msin_mstp(msin), mstp);
        }
    }

    #[test]
    fn msin_log_levels() {
        for mtin in [LOG_LEVEL_FATAL, LOG_LEVEL_ERROR, LOG_LEVEL_WARN,
                     LOG_LEVEL_INFO, LOG_LEVEL_DEBUG, LOG_LEVEL_VERBOSE] {
            let msin = build_msin(MESSAGE_TYPE_LOG, mtin);
            assert_eq!(msin_mtin(msin), mtin);
        }
    }

    #[test]
    fn tmsp2_roundtrip() {
        let ns = 1_700_000_000u64 * 1_000_000_000 + 123_456_789;
        let encoded = encode_tmsp2(ns);
        assert_eq!(decode_tmsp2(&encoded), ns);
    }

    #[test]
    fn tmsp2_zero() {
        let encoded = encode_tmsp2(0);
        assert_eq!(decode_tmsp2(&encoded), 0);
    }
}
