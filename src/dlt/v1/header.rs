use crate::dlt::error::ParseErrorKind;
use crate::dlt::protocol::*;

/// Parsed v1 header information.
pub struct ParsedHeader {
    pub htyp: u8,
    pub msin: u8,
    pub ecu: Option<[u8; 4]>,
    pub apid: Option<[u8; 4]>,
    pub ctid: Option<[u8; 4]>,
    pub session_id: Option<u32>,
    pub message_timestamp_ns: u64,
    pub message_type: u8,
    pub log_level: u8,
    /// Byte offset of the payload start within the message slice.
    pub payload_offset: usize,
    /// Payload length in bytes.
    pub payload_len: usize,
}

/// v1 standard header minimum size: HTYP(1) + MCNT(1) + LEN(2) = 4
const V1_STD_HEADER_MIN: usize = 4;

/// v1 extended header size: MSIN(1) + NOAR(1) + APID(4) + CTID(4) = 10
const V1_EXT_HEADER_SIZE: usize = 10;

/// Parse a v1 header from a message slice.
///
/// `msg` starts at the standard header (HTYP byte) and has the full message length.
/// Returns a specific `ParseErrorKind` when the message is malformed.
pub(super) fn parse_v1_header(msg: &[u8]) -> Result<ParsedHeader, ParseErrorKind> {
    debug_assert!(msg.len() >= V1_STD_HEADER_MIN);

    let htyp = msg[0];
    let len = msg.len();
    let mut offset: usize = V1_STD_HEADER_MIN;

    // Optional fields based on HTYP flags
    let ecu = if htyp_has_weid(htyp) {
        if offset + DLT_SIZE_WEID > len {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        let val: [u8; 4] = msg[offset..offset + 4].try_into().unwrap();
        offset += DLT_SIZE_WEID;
        Some(val)
    } else {
        None
    };

    let session_id = if htyp_has_wsid(htyp) {
        if offset + DLT_SIZE_WSID > len {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        let val = u32::from_be_bytes(msg[offset..offset + 4].try_into().unwrap());
        offset += DLT_SIZE_WSID;
        Some(val)
    } else {
        None
    };

    // Timestamp: 0.1ms ticks (u32) -> convert to nanoseconds (* 100_000)
    let message_timestamp_ns = if htyp_has_wtms(htyp) {
        if offset + DLT_SIZE_WTMS > len {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        let ticks = u32::from_be_bytes(msg[offset..offset + 4].try_into().unwrap());
        offset += DLT_SIZE_WTMS;
        (ticks as u64) * 100_000
    } else {
        0
    };

    // Extended header (if UEH flag set)
    let mut apid = None;
    let mut ctid = None;
    let mut message_type: u8 = 0;
    let mut log_level: u8 = 0;
    let mut msin_byte: u8 = 0;

    if htyp_has_ueh(htyp) {
        if offset + V1_EXT_HEADER_SIZE > len {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        msin_byte = msg[offset];
        // NOAR at offset+1 (skip)
        message_type = msin_mstp(msin_byte);
        log_level = msin_mtin(msin_byte);

        let apid_start = offset + 2;
        apid = Some(msg[apid_start..apid_start + 4].try_into().unwrap());

        let ctid_start = apid_start + 4;
        ctid = Some(msg[ctid_start..ctid_start + 4].try_into().unwrap());

        offset += V1_EXT_HEADER_SIZE;
    }

    let payload_offset = offset;
    let payload_len = len - payload_offset;

    Ok(ParsedHeader {
        htyp,
        msin: msin_byte,
        ecu,
        apid,
        ctid,
        session_id,
        message_timestamp_ns,
        message_type,
        log_level,
        payload_offset,
        payload_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a v1 message with the given flags and optional fields.
    fn build_v1_msg(
        ueh: bool,
        weid: bool,
        wsid: bool,
        wtms: bool,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut htyp: u8 = 1 << 5; // version = 1
        if ueh {
            htyp |= 0x01;
        }
        if weid {
            htyp |= 0x04;
        }
        if wsid {
            htyp |= 0x08;
        }
        if wtms {
            htyp |= 0x10;
        }

        let mut msg = Vec::new();
        msg.push(htyp);
        msg.push(0x42); // MCNT

        // We'll fix up LEN after building the full message
        msg.push(0x00); // LEN placeholder high byte
        msg.push(0x00); // LEN placeholder low byte

        if weid {
            msg.extend_from_slice(b"ECU1");
        }
        if wsid {
            msg.extend_from_slice(&0xAABBCCDDu32.to_be_bytes());
        }
        if wtms {
            // 1000 ticks = 100ms = 100_000_000 ns
            msg.extend_from_slice(&1000u32.to_be_bytes());
        }
        if ueh {
            // MSIN: MSTP=LOG(0x00), MTIN=WARN(0x03) -> (0x00 << 1) | (0x03 << 4) = 0x30
            msg.push(0x30);
            msg.push(0x02); // NOAR
            msg.extend_from_slice(b"APP1");
            msg.extend_from_slice(b"CTX1");
        }

        msg.extend_from_slice(payload);

        // Fix up LEN (total message length including header)
        let len = msg.len() as u16;
        msg[2] = (len >> 8) as u8;
        msg[3] = (len & 0xFF) as u8;

        msg
    }

    #[test]
    fn all_flags_set() {
        let payload = b"hello";
        let msg = build_v1_msg(true, true, true, true, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, Some(*b"ECU1"));
        assert_eq!(hdr.session_id, Some(0xAABBCCDD));
        assert_eq!(hdr.message_timestamp_ns, 1000 * 100_000);
        assert_eq!(hdr.apid, Some(*b"APP1"));
        assert_eq!(hdr.ctid, Some(*b"CTX1"));
        assert_eq!(hdr.message_type, 0x00); // LOG
        assert_eq!(hdr.log_level, 0x03); // WARN
        assert_eq!(hdr.payload_len, payload.len());
        assert_eq!(&msg[hdr.payload_offset..], payload);
    }

    #[test]
    fn no_flags_set() {
        let payload = b"data";
        let msg = build_v1_msg(false, false, false, false, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, None);
        assert_eq!(hdr.session_id, None);
        assert_eq!(hdr.message_timestamp_ns, 0);
        assert_eq!(hdr.apid, None);
        assert_eq!(hdr.ctid, None);
        assert_eq!(hdr.message_type, 0);
        assert_eq!(hdr.log_level, 0);
        assert_eq!(hdr.payload_len, payload.len());
        assert_eq!(&msg[hdr.payload_offset..], payload);
    }

    #[test]
    fn ueh_only() {
        let payload = b"ext";
        let msg = build_v1_msg(true, false, false, false, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, None);
        assert_eq!(hdr.session_id, None);
        assert_eq!(hdr.message_timestamp_ns, 0);
        assert_eq!(hdr.apid, Some(*b"APP1"));
        assert_eq!(hdr.ctid, Some(*b"CTX1"));
        assert_eq!(hdr.payload_len, payload.len());
    }

    #[test]
    fn weid_only() {
        let payload = b"ecu";
        let msg = build_v1_msg(false, true, false, false, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, Some(*b"ECU1"));
        assert_eq!(hdr.session_id, None);
        assert_eq!(hdr.message_timestamp_ns, 0);
        assert_eq!(hdr.apid, None);
        assert_eq!(hdr.payload_len, payload.len());
    }

    #[test]
    fn wsid_only() {
        let payload = b"sid";
        let msg = build_v1_msg(false, false, true, false, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, None);
        assert_eq!(hdr.session_id, Some(0xAABBCCDD));
        assert_eq!(hdr.message_timestamp_ns, 0);
        assert_eq!(hdr.payload_len, payload.len());
    }

    #[test]
    fn wtms_only() {
        let payload = b"ts";
        let msg = build_v1_msg(false, false, false, true, payload);
        let hdr = parse_v1_header(&msg).unwrap();

        assert_eq!(hdr.ecu, None);
        assert_eq!(hdr.session_id, None);
        assert_eq!(hdr.message_timestamp_ns, 100_000_000); // 1000 * 100_000
        assert_eq!(hdr.payload_len, payload.len());
    }

    #[test]
    fn payload_offset_no_extra_no_extended() {
        let msg = build_v1_msg(false, false, false, false, b"AB");
        let hdr = parse_v1_header(&msg).unwrap();
        // Standard header only: 4 bytes
        assert_eq!(hdr.payload_offset, 4);
        assert_eq!(hdr.payload_len, 2);
    }

    #[test]
    fn payload_offset_all_extras_with_extended() {
        let msg = build_v1_msg(true, true, true, true, b"XYZ");
        let hdr = parse_v1_header(&msg).unwrap();
        // Std(4) + WEID(4) + WSID(4) + WTMS(4) + Ext(10) = 26
        assert_eq!(hdr.payload_offset, 26);
        assert_eq!(hdr.payload_len, 3);
    }
}
