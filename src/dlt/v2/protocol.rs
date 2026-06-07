#![allow(dead_code)]

/// Base header minimum size: HTYP2(4) + MCNT(1) + LEN(2) = 7.
pub const BASE_HEADER_MIN_SIZE: usize = 7;

// HTYP2 layout constants.
pub const HTYP2_CNTI_SHIFT: u32 = 24;
pub const HTYP2_CNTI_MASK: u32 = 0x03 << HTYP2_CNTI_SHIFT;
pub const CNTI_VERBOSE: u8 = 0x00;
pub const CNTI_NON_VERBOSE: u8 = 0x01;
pub const CNTI_CONTROL: u8 = 0x02;

pub const HTYP2_WEID: u32 = 1 << 26;
pub const HTYP2_WACID: u32 = 1 << 27;
pub const HTYP2_WSID: u32 = 1 << 28;

pub const HTYP2_VERS_SHIFT: u32 = 29;
pub const HTYP2_VERS_MASK: u32 = 0x07 << HTYP2_VERS_SHIFT;
pub const PROTOCOL_VERSION_2: u8 = 2;

pub const HTYP2_WSFLN: u32 = 1 << 16;
pub const HTYP2_WTGS: u32 = 1 << 17;
pub const HTYP2_WPVL: u32 = 1 << 18;
pub const HTYP2_WSGM: u32 = 1 << 19;

// MSIN layout: bits 1..3 = MSTP, bits 4..7 = MTIN.
pub const MESSAGE_TYPE_LOG: u8 = 0x00;
pub const MESSAGE_TYPE_TRACE: u8 = 0x01;
pub const MESSAGE_TYPE_NETWORK: u8 = 0x02;
pub const MESSAGE_TYPE_CONTROL: u8 = 0x03;

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

#[inline]
pub fn msin_mstp(msin: u8) -> u8 {
    (msin >> 1) & 0x07
}

#[inline]
pub fn msin_mtin(msin: u8) -> u8 {
    (msin >> 4) & 0x0F
}

#[inline]
pub fn build_msin(mstp: u8, mtin: u8) -> u8 {
    ((mstp & 0x07) << 1) | ((mtin & 0x0F) << 4)
}

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

/// Encode a nanosecond timestamp into a 9-byte TMSP2 field.
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

/// Decode a 9-byte TMSP2 field into nanoseconds.
pub fn decode_tmsp2(tmsp2: &[u8; 9]) -> u64 {
    let nanoseconds = u32::from_be_bytes(tmsp2[0..4].try_into().unwrap()) & 0x7FFF_FFFF;
    let seconds = ((tmsp2[4] as u64) << 32)
        | ((tmsp2[5] as u64) << 24)
        | ((tmsp2[6] as u64) << 16)
        | ((tmsp2[7] as u64) << 8)
        | (tmsp2[8] as u64);
    seconds
        .saturating_mul(1_000_000_000)
        .saturating_add(nanoseconds as u64)
}
