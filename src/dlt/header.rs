use super::protocol::*;

/// Parsed v2 header information.
pub struct ParsedHeader {
    pub htyp2: u32,
    pub apid: Option<[u8; 4]>,
    pub ctid: Option<[u8; 4]>,
    /// Byte offset of the payload start within the message slice.
    pub payload_offset: usize,
    /// Payload length in bytes.
    pub payload_len: usize,
}

/// Parse a v2 base header + extension header from a message slice.
///
/// `msg` starts at the base header (HTYP2) and has length = LEN.
/// Returns `None` if the message is malformed or too short.
pub fn parse_v2_header(msg: &[u8]) -> Option<ParsedHeader> {
    if msg.len() < BASE_HEADER_MIN_SIZE {
        return None;
    }

    let htyp2 = u32::from_be_bytes(msg[0..4].try_into().ok()?);
    if htyp2_version(htyp2) != PROTOCOL_VERSION_2 {
        return None;
    }

    let len = u16::from_be_bytes(msg[5..7].try_into().ok()?) as usize;
    let mut offset = 7; // past HTYP2(4) + MCNT(1) + LEN(2)

    let cnti = htyp2_cnti(htyp2);

    // MSIN + NOAR for verbose or control messages
    if cnti == CNTI_VERBOSE || cnti == CNTI_CONTROL {
        if offset + 2 > msg.len() {
            return None;
        }
        offset += 2;
    }

    // TMSP2 (9 bytes) for data messages (verbose or non-verbose)
    if cnti == CNTI_VERBOSE || cnti == CNTI_NON_VERBOSE {
        if offset + 9 > msg.len() {
            return None;
        }
        offset += 9;
    }

    // MSID (4 bytes) for non-verbose
    if cnti == CNTI_NON_VERBOSE {
        if offset + 4 > msg.len() {
            return None;
        }
        offset += 4;
    }

    // --- Extension Header ---
    let mut apid = None;
    let mut ctid = None;

    // ECU ID (WEID) — length-prefixed
    if htyp2_has_weid(htyp2) {
        if offset >= msg.len() {
            return None;
        }
        let ecu_len = msg[offset] as usize;
        offset += 1 + ecu_len;
        if offset > msg.len() {
            return None;
        }
    }

    // APID + CTID (WACID) — each length-prefixed
    if htyp2_has_wacid(htyp2) {
        // APID
        if offset >= msg.len() {
            return None;
        }
        let apid_len = msg[offset] as usize;
        offset += 1;
        if offset + apid_len > msg.len() {
            return None;
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
            return None;
        }
        let ctid_len = msg[offset] as usize;
        offset += 1;
        if offset + ctid_len > msg.len() {
            return None;
        }
        if ctid_len > 0 {
            let mut buf = [0u8; 4];
            let n = ctid_len.min(4);
            buf[..n].copy_from_slice(&msg[offset..offset + n]);
            ctid = Some(buf);
        }
        offset += ctid_len;
    }

    // WSID — fixed 4 bytes, no length prefix
    if htyp2_has_wsid(htyp2) {
        if offset + 4 > msg.len() {
            return None;
        }
        offset += 4;
    }

    // Remaining extension header fields (WSFLN, WTGS, WPVL, WSGM) are
    // skipped for the tracer bullet — full parsing is issue #46.

    let payload_offset = offset;
    let payload_len = len.saturating_sub(payload_offset);

    Some(ParsedHeader {
        htyp2,
        apid,
        ctid,
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

        msg.push(0x00); // MSIN
        msg.push(1); // NOAR = 1

        // TMSP2 (9 bytes of zeros)
        msg.extend_from_slice(&[0u8; 9]);

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
        assert_eq!(header.payload_len, 4);

        let payload = &msg[header.payload_offset..header.payload_offset + header.payload_len];
        assert_eq!(payload, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn rejects_v1_header() {
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, 1);
        let mut msg = vec![0u8; 9];
        msg[0..4].copy_from_slice(&htyp2.to_be_bytes());
        msg[5..7].copy_from_slice(&9u16.to_be_bytes());

        assert!(parse_v2_header(&msg).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(parse_v2_header(&[0; 3]).is_none());
    }
}
