#![allow(dead_code)]

/// HTYP bitfield definitions.
const HTYP_UEH: u8 = 0x01; // use extended header
const HTYP_MSBF: u8 = 0x02; // payload byte order: most-significant-byte first
const HTYP_WEID: u8 = 0x04; // with ECU ID
const HTYP_WSID: u8 = 0x08; // with session ID
const HTYP_WTMS: u8 = 0x10; // with timestamp

pub const STD_HEADER_MIN: usize = 4; // HTYP(1) + MCNT(1) + LEN(2)
pub const EXT_HEADER_SIZE: usize = 10; // MSIN(1) + NOAR(1) + APID(4) + CTID(4)

/// Standard header field sizes.
pub const SIZE_WEID: usize = 4;
pub const SIZE_WSID: usize = 4;
pub const SIZE_WTMS: usize = 4;

/// Extended header field sizes.
pub const SIZE_APID: usize = 4;
pub const SIZE_CTID: usize = 4;

// MSIN layout: bits 1..3 = MSTP, bits 4..7 = MTIN.
// Message Type
pub const MESSAGE_TYPE_LOG: u8 = 0x00;
pub const MESSAGE_TYPE_TRACE: u8 = 0x01;
pub const MESSAGE_TYPE_NETWORK: u8 = 0x02;
pub const MESSAGE_TYPE_CONTROL: u8 = 0x03;

// Message Type Info
pub const LOG_LEVEL_FATAL: u8 = 0x01;
pub const LOG_LEVEL_ERROR: u8 = 0x02;
pub const LOG_LEVEL_WARN: u8 = 0x03;
pub const LOG_LEVEL_INFO: u8 = 0x04;
pub const LOG_LEVEL_DEBUG: u8 = 0x05;
pub const LOG_LEVEL_VERBOSE: u8 = 0x06;

pub const TRACE_VARIABLE: u8 = 0x01;
pub const TRACE_FUNCTION_IN: u8 = 0x02;
pub const TRACE_FUNCTION_OUT: u8 = 0x03;
pub const TRACE_STATE: u8 = 0x04;
pub const TRACE_VFB: u8 = 0x05;

pub const NETWORK_IPC: u8 = 0x01;
pub const NETWORK_CAN: u8 = 0x02;
pub const NETWORK_FLEXRAY: u8 = 0x03;
pub const NETWORK_MOST: u8 = 0x04;
pub const NETWORK_ETHERNET: u8 = 0x05;
pub const NETWORK_SOMEIP: u8 = 0x06;

pub const CONTROL_REQUEST: u8 = 0x01;
pub const CONTROL_RESPONSE: u8 = 0x02;

#[inline]
pub fn htyp_has_ueh(htyp: u8) -> bool {
    htyp & HTYP_UEH != 0
}

#[inline]
pub fn htyp_has_msbf(htyp: u8) -> bool {
    htyp & HTYP_MSBF != 0
}

#[inline]
pub fn htyp_has_weid(htyp: u8) -> bool {
    htyp & HTYP_WEID != 0
}

#[inline]
pub fn htyp_has_wsid(htyp: u8) -> bool {
    htyp & HTYP_WSID != 0
}

#[inline]
pub fn htyp_has_wtms(htyp: u8) -> bool {
    htyp & HTYP_WTMS != 0
}

#[inline]
pub fn msin_mstp(msin: u8) -> u8 {
    (msin >> 1) & 0x07
}

#[inline]
pub fn msin_mtin(msin: u8) -> u8 {
    (msin >> 4) & 0x0F
}

#[inline]
pub fn msin_is_verb(msin: u8) -> bool {
    (msin & 0x01) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msin_extracts_message_type_and_info() {
        // MSIN: MSTP=LOG(0), MTIN=WARN(3), VERB=0 -> 0x30
        let msin = 0x30;
        assert_eq!(msin_mstp(msin), MESSAGE_TYPE_LOG);
        assert_eq!(msin_mtin(msin), LOG_LEVEL_WARN);
    }

    #[test]
    fn msin_extracts_other_message_families() {
        // MSTP=NETWORK(2), MTIN=SOMEIP(6), VERB=1
        let msin = (MESSAGE_TYPE_NETWORK << 1) | (NETWORK_SOMEIP << 4) | 0x01;
        assert_eq!(msin_mstp(msin), MESSAGE_TYPE_NETWORK);
        assert_eq!(msin_mtin(msin), NETWORK_SOMEIP);
        assert!(msin_is_verb(msin));
    }

    #[test]
    fn msin_extracts_control_type_info() {
        // MSTP=CONTROL(3), MTIN=RESPONSE(2), VERB=0
        let msin = (MESSAGE_TYPE_CONTROL << 1) | (CONTROL_RESPONSE << 4);
        assert_eq!(msin_mstp(msin), MESSAGE_TYPE_CONTROL);
        assert_eq!(msin_mtin(msin), CONTROL_RESPONSE);
        assert!(!msin_is_verb(msin));
    }
}
