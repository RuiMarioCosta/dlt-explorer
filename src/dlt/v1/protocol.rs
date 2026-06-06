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
pub const _MESSAGE_TYPE_LOG: u8 = 0x00;
pub const _MESSAGE_TYPE_TRACE: u8 = 0x01;
pub const _MESSAGE_TYPE_NETWORK: u8 = 0x02;
pub const MESSAGE_TYPE_CONTROL: u8 = 0x03;

pub const _LOG_FATAL: u8 = 0x01;
pub const _LOG_ERROR: u8 = 0x02;
pub const _LOG_WARN: u8 = 0x03;
pub const _LOG_INFO: u8 = 0x04;
pub const _LOG_DEBUG: u8 = 0x05;
pub const _LOG_VERBOSE: u8 = 0x06;

pub const _CONTROL_REQUEST: u8 = 0x01;
pub const _CONTROL_RESPONSE: u8 = 0x02;

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
