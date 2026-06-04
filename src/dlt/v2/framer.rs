use memchr::memmem::Finder;

use crate::dlt::error::{ParseError, ParseErrorKind};
use crate::dlt::protocol::*;

/// A located DLT v2 frame within memory-mapped data.
pub struct Frame {
    /// Nanosecond-resolution timestamp from the storage header.
    pub storage_timestamp_ns: u64,
    /// Byte offset of the base header start within the data slice.
    pub msg_start: usize,
    /// Message length from the base header LEN field.
    pub msg_len: usize,
}

/// Output of v2 frame scanning.
pub struct ScanOutput {
    pub frames: Vec<Frame>,
    pub errors: Vec<ParseError>,
    /// Default storage header ECU for this file.
    pub default_storage_ecu: Option<[u8; 4]>,
    /// Sparse per-frame storage ECU overrides: (frame_index, ecu).
    pub storage_ecu_overrides: Vec<(usize, [u8; 4])>,
}

/// Scan `data` for DLT v2 frames, returning one `Frame` per valid v2 message
/// found, plus any `ParseError`s for malformed frames encountered along the way.
///
/// On error the scanner advances to the next `DLT\x01` marker.
pub fn scan_frames(data: &[u8], file_index: u16) -> ScanOutput {
    let finder = Finder::new(DLT_STORAGE_HEADER_PATTERN);
    let mut frames = Vec::new();
    let mut errors = Vec::new();
    let mut default_storage_ecu = None;
    let mut storage_ecu_overrides = Vec::new();
    let mut search_start = 0;

    while let Some(rel_pos) = finder.find(&data[search_start..]) {
        let pos = search_start + rel_pos;
        let storage_end = pos + STORAGE_HEADER_SIZE;

        // Need at least the storage header + minimum base header to proceed
        if storage_end + BASE_HEADER_MIN_SIZE > data.len() {
            errors.push(ParseError {
                file_index,
                byte_offset: pos as u64,
                kind: ParseErrorKind::Truncated,
            });
            break;
        }

        // Parse storage header fields (LE, per dlt-daemon DltStorageHeader)
        let seconds = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());
        let microseconds = u32::from_le_bytes(data[pos + 8..pos + 12].try_into().unwrap());
        let mut ecu = [0u8; 4];
        ecu.copy_from_slice(&data[pos + 12..pos + 16]);

        let timestamp_ns = seconds as u64 * 1_000_000_000 + microseconds as u64 * 1_000;

        // Base header starts immediately after the storage header
        let msg_start = storage_end;
        let htyp2 = u32::from_be_bytes(data[msg_start..msg_start + 4].try_into().unwrap());

        // Report non-v2 messages
        let version = htyp2_version(htyp2);
        if version != PROTOCOL_VERSION_2 {
            errors.push(ParseError {
                file_index,
                byte_offset: pos as u64,
                kind: ParseErrorKind::InvalidVersion { found: version },
            });
            search_start = pos + 1;
            continue;
        }

        let len =
            u16::from_be_bytes(data[msg_start + 5..msg_start + 7].try_into().unwrap());

        if (len as usize) < BASE_HEADER_MIN_SIZE {
            errors.push(ParseError {
                file_index,
                byte_offset: pos as u64,
                kind: ParseErrorKind::LengthMismatch {
                    declared: len,
                    available: data.len() - msg_start,
                },
            });
            search_start = pos + 1;
            continue;
        }

        if msg_start + len as usize > data.len() {
            errors.push(ParseError {
                file_index,
                byte_offset: pos as u64,
                kind: ParseErrorKind::LengthMismatch {
                    declared: len,
                    available: (data.len() - msg_start),
                },
            });
            search_start = pos + 1;
            continue;
        }

        frames.push(Frame {
            storage_timestamp_ns: timestamp_ns,
            msg_start,
            msg_len: len as usize,
        });

        let frame_idx = frames.len() - 1;
        match default_storage_ecu {
            None => default_storage_ecu = Some(ecu),
            Some(default) if default != ecu => storage_ecu_overrides.push((frame_idx, ecu)),
            _ => {}
        }

        // Advance past this message
        search_start = msg_start + len as usize;
    }

    ScanOutput {
        frames,
        errors,
        default_storage_ecu,
        storage_ecu_overrides,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal v2 frame (storage header + base header only, no payload).
    fn minimal_v2_frame() -> Vec<u8> {
        let mut buf = Vec::new();

        // Storage header (16 bytes)
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&100u32.to_le_bytes()); // seconds
        buf.extend_from_slice(&500u32.to_le_bytes()); // microseconds
        buf.extend_from_slice(b"ECU1"); // ECU ID

        // Base header: HTYP2 (verbose, VERS=2) + MCNT + LEN
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
        buf.extend_from_slice(&htyp2.to_be_bytes());
        buf.push(0); // MCNT
        let len: u16 = 9; // base header (7) + MSIN(1) + NOAR(1)
        buf.extend_from_slice(&len.to_be_bytes());
        buf.push(0); // MSIN
        buf.push(0); // NOAR

        buf
    }

    #[test]
    fn finds_single_v2_frame() {
        let data = minimal_v2_frame();
        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 0);
        assert_eq!(
            out.frames[0].storage_timestamp_ns,
            100 * 1_000_000_000 + 500 * 1_000
        );
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
        assert!(out.storage_ecu_overrides.is_empty());
    }

    #[test]
    fn skips_v1_frame() {
        let mut buf = Vec::new();
        // Storage header (16 bytes)
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");

        // Base header with VERS=1
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, 1);
        buf.extend_from_slice(&htyp2.to_be_bytes());
        buf.push(0); // MCNT
        buf.extend_from_slice(&9u16.to_be_bytes());
        buf.push(0);
        buf.push(0);

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::InvalidVersion { found: 1 });
    }

    #[test]
    fn empty_input() {
        let out = scan_frames(&[], 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 0);
        assert_eq!(out.default_storage_ecu, None);
        assert!(out.storage_ecu_overrides.is_empty());
    }

    #[test]
    fn truncated_storage_header() {
        let out = scan_frames(b"DLT\x01too_short", 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn truncated_mid_header_no_panic() {
        // File ends mid-header (storage header present but base header truncated)
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");
        // Only 3 bytes of base header instead of 7
        buf.extend_from_slice(&[0x00, 0x00, 0x00]);

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn garbage_between_valid_messages() {
        let mut data = minimal_v2_frame();
        // Insert garbage with a DLT\x01 marker but invalid version
        data.extend_from_slice(b"DLT\x01");
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(b"ECU1");
        let bad_htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, 3); // invalid version
        data.extend_from_slice(&bad_htyp2.to_be_bytes());
        data.push(0);
        data.extend_from_slice(&9u16.to_be_bytes());
        data.push(0);
        data.push(0);
        // Second valid message
        data.extend_from_slice(&minimal_v2_frame());

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 2);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::InvalidVersion { found: 3 });
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
    }

    #[test]
    fn length_mismatch_declared_exceeds_available() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
        buf.extend_from_slice(&htyp2.to_be_bytes());
        buf.push(0);
        // Declared length far exceeds available data
        buf.extend_from_slice(&500u16.to_be_bytes());
        buf.push(0);
        buf.push(0);

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        match &out.errors[0].kind {
            ParseErrorKind::LengthMismatch { declared, .. } => assert_eq!(*declared, 500),
            other => panic!("expected LengthMismatch, got {:?}", other),
        }
    }

    #[test]
    fn marker_in_payload_not_confused_as_frame() {
        // Build a valid frame whose payload contains DLT\x01
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&100u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");

        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, PROTOCOL_VERSION_2);
        buf.extend_from_slice(&htyp2.to_be_bytes());
        buf.push(0); // MCNT
        // payload = DLT\x01 (4 bytes), so total len = 7 + 2 + 4 = 13
        let len: u16 = 13;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.push(0); // MSIN
        buf.push(0); // NOAR
        buf.extend_from_slice(b"DLT\x01"); // fake marker in payload

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 0);
    }

    #[test]
    fn file_ending_mid_message_after_valid() {
        let mut data = minimal_v2_frame();
        // Append a truncated storage header (only pattern + partial data)
        data.extend_from_slice(b"DLT\x01");
        data.extend_from_slice(&[0u8; 5]); // not enough for full storage + base header

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn mixed_storage_ecu_uses_sparse_overrides() {
        let mut data = minimal_v2_frame();

        let mut second = minimal_v2_frame();
        second[12..16].copy_from_slice(b"ECU2");
        data.extend_from_slice(&second);

        data.extend_from_slice(&minimal_v2_frame());

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 3);
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
        assert_eq!(out.storage_ecu_overrides.len(), 1);
        assert_eq!(out.storage_ecu_overrides[0], (1, *b"ECU2"));
    }
}
