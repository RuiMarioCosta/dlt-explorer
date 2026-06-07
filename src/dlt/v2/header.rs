use super::protocol::*;
use crate::dlt::error::ParseErrorKind;

/// Parsed v2 header information.
pub struct ParsedHeader {
    pub htyp2: u32,
    pub apid: Option<[u8; 4]>,
    pub ctid: Option<[u8; 4]>,
    pub ecu: Option<[u8; 4]>,
    pub session_id: Option<u32>,
    pub message_timestamp_ns: u64,
    pub message_type: u8,
    pub message_type_info: u8,
    /// Byte offset of the payload start within the message slice.
    pub payload_offset: usize,
    /// Payload length in bytes.
    pub payload_len: usize,
}

/// Parse a v2 base header + extension header from a message slice.
///
/// `msg` starts at the base header (HTYP2) and has length = LEN.
/// Returns a specific `ParseErrorKind` when extension parsing fails.
pub(super) fn parse_v2_header(msg: &[u8]) -> Result<ParsedHeader, ParseErrorKind> {
    debug_assert!(msg.len() >= BASE_HEADER_MIN_SIZE);

    let htyp2 = u32::from_be_bytes(msg[0..4].try_into().unwrap());
    let len = msg.len();
    let mut offset = 7; // past HTYP2(4) + MCNT(1) + LEN(2)

    let cnti = htyp2_cnti(htyp2);

    // MSIN + NOAR for verbose or control messages
    let mut message_type: u8 = 0;
    let mut message_type_info: u8 = 0;

    if cnti == CNTI_VERBOSE || cnti == CNTI_CONTROL {
        if offset + 2 > msg.len() {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        let msin = msg[offset];
        message_type = msin_mstp(msin);
        message_type_info = msin_mtin(msin);
        offset += 2;
    }

    // TMSP2 (9 bytes) for data messages (verbose or non-verbose)
    let mut message_timestamp_ns: u64 = 0;
    if cnti == CNTI_VERBOSE || cnti == CNTI_NON_VERBOSE {
        if offset + 9 > msg.len() {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        let tmsp2: [u8; 9] = msg[offset..offset + 9].try_into().unwrap();
        message_timestamp_ns = decode_tmsp2(&tmsp2);
        offset += 9;
    }

    // MSID (4 bytes) for non-verbose
    if cnti == CNTI_NON_VERBOSE {
        if offset + 4 > msg.len() {
            return Err(ParseErrorKind::InvalidStandardHeader);
        }
        offset += 4;
    }

    // --- Extension Header ---
    let mut apid = None;
    let mut ctid = None;
    let mut ecu = None;
    let mut session_id = None;

    // ECU ID (WEID) — length-prefixed
    if htyp2_has_weid(htyp2) {
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        let ecu_len = msg[offset] as usize;
        offset += 1;
        if offset + ecu_len > msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        if ecu_len > 0 {
            let mut buf = [0u8; 4];
            let n = ecu_len.min(4);
            buf[..n].copy_from_slice(&msg[offset..offset + n]);
            ecu = Some(buf);
        }
        offset += ecu_len;
    }

    // APID + CTID (WACID) — each length-prefixed
    if htyp2_has_wacid(htyp2) {
        // APID
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        let apid_len = msg[offset] as usize;
        offset += 1;
        if offset + apid_len > msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        if apid_len > 0 {
            let mut buf = [0u8; 4];
            let n = apid_len.min(4);
            buf[..n].copy_from_slice(&msg[offset..offset + n]);
            apid = Some(buf);
        }
        offset += apid_len;

        // CTID
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        let ctid_len = msg[offset] as usize;
        offset += 1;
        if offset + ctid_len > msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        if ctid_len > 0 {
            let mut buf = [0u8; 4];
            let n = ctid_len.min(4);
            buf[..n].copy_from_slice(&msg[offset..offset + n]);
            ctid = Some(buf);
        }
        offset += ctid_len;
    }

    // WSID — fixed 4 bytes
    if htyp2_has_wsid(htyp2) {
        if offset + 4 > msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        session_id = Some(u32::from_be_bytes(
            msg[offset..offset + 4].try_into().unwrap(),
        ));
        offset += 4;
    }

    // WSFLN — source filename (length-prefixed) + line number (u32)
    if htyp2_has_wsfln(htyp2) {
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        let fina_len = msg[offset] as usize;
        offset += 1 + fina_len;
        if offset + 4 > msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        offset += 4; // LINR u32
    }

    // WTGS — tags: NOTG (u8) + per tag: length (u8) + name bytes
    if htyp2_has_wtgs(htyp2) {
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        let notg = msg[offset] as usize;
        offset += 1;
        for _ in 0..notg {
            if offset >= msg.len() {
                return Err(ParseErrorKind::InvalidExtensionField);
            }
            let tag_len = msg[offset] as usize;
            offset += 1 + tag_len;
            if offset > msg.len() {
                return Err(ParseErrorKind::InvalidExtensionField);
            }
        }
    }

    // WPVL — privacy level (u8)
    if htyp2_has_wpvl(htyp2) {
        if offset >= msg.len() {
            return Err(ParseErrorKind::InvalidExtensionField);
        }
        offset += 1;
    }

    // WSGM — segmentation (skip for now, not fully specified)
    if htyp2_has_wsgm(htyp2) {
        // Segmentation details are not yet specified in the issue;
        // skip gracefully if present. We can't know the exact length,
        // so we leave offset as-is and let payload_len absorb it.
    }

    let payload_offset = offset;
    let payload_len = len.saturating_sub(payload_offset);

    Ok(ParsedHeader {
        htyp2,
        apid,
        ctid,
        ecu,
        session_id,
        message_timestamp_ns,
        message_type,
        message_type_info,
        payload_offset,
        payload_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_verbose_with_wacid() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, true, false, PROTOCOL_VERSION_2);

        let mut msg = Vec::new();
        msg.extend_from_slice(&htyp2.to_be_bytes()); // HTYP2
        msg.push(0); // MCNT

        // We'll fill LEN after computing total size
        let len_pos = msg.len();
        msg.extend_from_slice(&0u16.to_be_bytes()); // placeholder

        msg.push(build_msin(MESSAGE_TYPE_LOG, LOG_LEVEL_INFO)); // MSIN
        msg.push(1); // NOAR = 1

        // TMSP2 (9 bytes)
        let ts_ns = 1000u64 * 1_000_000_000 + 500_000;
        msg.extend_from_slice(&encode_tmsp2(ts_ns));

        // Extension header: APID
        msg.push(4); // APID length
        msg.extend_from_slice(b"APP1");

        // CTID
        msg.push(4); // CTID length
        msg.extend_from_slice(b"CTX1");

        // Payload: 4 bytes of dummy data
        msg.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        // Patch LEN
        let len = msg.len() as u16;
        msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

        let header = parse_v2_header(&msg).unwrap();
        assert_eq!(header.apid, Some(*b"APP1"));
        assert_eq!(header.ctid, Some(*b"CTX1"));
        assert_eq!(header.message_type, MESSAGE_TYPE_LOG);
        assert_eq!(header.message_type_info, LOG_LEVEL_INFO);
        assert_eq!(header.message_timestamp_ns, ts_ns);
        assert_eq!(header.payload_len, 4);

        let payload = &msg[header.payload_offset..header.payload_offset + header.payload_len];
        assert_eq!(payload, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_all_extension_fields_present() {
        let htyp2 = build_htyp2_full(
            CNTI_VERBOSE,
            true,
            true,
            true,
            PROTOCOL_VERSION_2,
            true,  // WSFLN
            true,  // WTGS
            true,  // WPVL
            false, // WSGM
        );

        let mut msg = Vec::new();
        msg.extend_from_slice(&htyp2.to_be_bytes());
        msg.push(0); // MCNT
        let len_pos = msg.len();
        msg.extend_from_slice(&0u16.to_be_bytes()); // placeholder LEN

        msg.push(build_msin(MESSAGE_TYPE_TRACE, LOG_LEVEL_WARN)); // MSIN
        msg.push(0); // NOAR

        let ts_ns = 42u64 * 1_000_000_000 + 999_999_999;
        msg.extend_from_slice(&encode_tmsp2(ts_ns)); // TMSP2

        // WEID: ECU ID
        msg.push(4);
        msg.extend_from_slice(b"ECU2");

        // WACID: APID + CTID
        msg.push(4);
        msg.extend_from_slice(b"AP01");
        msg.push(4);
        msg.extend_from_slice(b"CT01");

        // WSID: session ID = 0x12345678
        msg.extend_from_slice(&0x12345678u32.to_be_bytes());

        // WSFLN: source filename + line number
        msg.push(5); // filename length
        msg.extend_from_slice(b"a.cpp");
        msg.extend_from_slice(&42u32.to_be_bytes()); // line number

        // WTGS: 1 tag
        msg.push(1); // NOTG = 1
        msg.push(3); // tag length
        msg.extend_from_slice(b"foo");

        // WPVL: privacy level
        msg.push(7);

        // Payload
        msg.extend_from_slice(&[0xFF]);

        // Patch LEN
        let len = msg.len() as u16;
        msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

        let header = parse_v2_header(&msg).unwrap();
        assert_eq!(header.ecu, Some(*b"ECU2"));
        assert_eq!(header.apid, Some(*b"AP01"));
        assert_eq!(header.ctid, Some(*b"CT01"));
        assert_eq!(header.session_id, Some(0x12345678));
        assert_eq!(header.message_timestamp_ns, ts_ns);
        assert_eq!(header.message_type, MESSAGE_TYPE_TRACE);
        assert_eq!(header.message_type_info, LOG_LEVEL_WARN);
        assert_eq!(header.payload_len, 1);
    }

    #[test]
    fn parse_all_flags_cleared() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);

        let mut msg = Vec::new();
        msg.extend_from_slice(&htyp2.to_be_bytes());
        msg.push(0);
        let len_pos = msg.len();
        msg.extend_from_slice(&0u16.to_be_bytes());

        msg.push(build_msin(MESSAGE_TYPE_LOG, LOG_LEVEL_DEBUG));
        msg.push(0); // NOAR

        msg.extend_from_slice(&encode_tmsp2(0)); // TMSP2

        let len = msg.len() as u16;
        msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

        let header = parse_v2_header(&msg).unwrap();
        assert_eq!(header.ecu, None);
        assert_eq!(header.apid, None);
        assert_eq!(header.ctid, None);
        assert_eq!(header.session_id, None);
        assert_eq!(header.message_type, MESSAGE_TYPE_LOG);
        assert_eq!(header.message_type_info, LOG_LEVEL_DEBUG);
    }

    #[test]
    fn unknown_extension_field_skipped() {
        // Simulate a future extension by appending extra bytes after WPVL
        // but before payload. The parser should treat them as payload.
        let htyp2 = build_htyp2_full(
            CNTI_VERBOSE,
            false,
            false,
            false,
            PROTOCOL_VERSION_2,
            false,
            false,
            true,
            false, // only WPVL
        );

        let mut msg = Vec::new();
        msg.extend_from_slice(&htyp2.to_be_bytes());
        msg.push(0);
        let len_pos = msg.len();
        msg.extend_from_slice(&0u16.to_be_bytes());

        msg.push(build_msin(MESSAGE_TYPE_LOG, LOG_LEVEL_INFO));
        msg.push(0);
        msg.extend_from_slice(&encode_tmsp2(0));

        msg.push(3); // WPVL = 3

        // Payload
        msg.extend_from_slice(&[0xAB, 0xCD]);

        let len = msg.len() as u16;
        msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

        let header = parse_v2_header(&msg).unwrap();
        // Parser succeeds — unknown fields would just be part of payload bytes
        assert_eq!(header.payload_len, 2);
    }

    #[test]
    fn message_types_from_msin() {
        for (mstp, expected_name) in [
            (MESSAGE_TYPE_LOG, "LOG"),
            (MESSAGE_TYPE_TRACE, "TRACE"),
            (MESSAGE_TYPE_NETWORK, "NETWORK"),
            (MESSAGE_TYPE_CONTROL, "CONTROL"),
        ] {
            let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
            let mut msg = Vec::new();
            msg.extend_from_slice(&htyp2.to_be_bytes());
            msg.push(0);
            let len_pos = msg.len();
            msg.extend_from_slice(&0u16.to_be_bytes());

            msg.push(build_msin(mstp, 0));
            msg.push(0);
            msg.extend_from_slice(&encode_tmsp2(0));

            let len = msg.len() as u16;
            msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

            let header = parse_v2_header(&msg).unwrap();
            assert_eq!(header.message_type, mstp, "failed for {expected_name}");
        }
    }

    #[test]
    fn message_type_info_from_msin() {
        for (mtin, label) in [
            (LOG_LEVEL_FATAL, "FATAL"),
            (LOG_LEVEL_ERROR, "ERROR"),
            (LOG_LEVEL_WARN, "WARN"),
            (LOG_LEVEL_INFO, "INFO"),
            (LOG_LEVEL_DEBUG, "DEBUG"),
            (LOG_LEVEL_VERBOSE, "VERBOSE"),
        ] {
            let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
            let mut msg = Vec::new();
            msg.extend_from_slice(&htyp2.to_be_bytes());
            msg.push(0);
            let len_pos = msg.len();
            msg.extend_from_slice(&0u16.to_be_bytes());

            msg.push(build_msin(MESSAGE_TYPE_LOG, mtin));
            msg.push(0);
            msg.extend_from_slice(&encode_tmsp2(0));

            let len = msg.len() as u16;
            msg[len_pos..len_pos + 2].copy_from_slice(&len.to_be_bytes());

            let header = parse_v2_header(&msg).unwrap();
            assert_eq!(header.message_type_info, mtin, "failed for {label}");
        }
    }
}
