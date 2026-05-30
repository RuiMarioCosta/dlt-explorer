// AUTOSAR PRS v2 protocol constants and bitfield helpers

/// DLT storage header delimiter
pub const DLT_STORAGE_HEADER_PATTERN: &[u8] = b"DLT\x01";

// Storage header sizes
pub const STORAGE_HEADER_PATTERN_SIZE: usize = 4;
pub const STORAGE_HEADER_TMSP2_SIZE: usize = 9; // seconds u32 LE + nanoseconds u32 LE + flags u8
pub const STORAGE_HEADER_ECU_SIZE: usize = 4;
pub const STORAGE_HEADER_SIZE: usize =
    STORAGE_HEADER_PATTERN_SIZE + STORAGE_HEADER_TMSP2_SIZE + STORAGE_HEADER_ECU_SIZE; // 17

/// Base header minimum size: HTYP2(4) + MCNT(1) + LEN(2) = 7
pub const BASE_HEADER_MIN_SIZE: usize = 7;

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
}
