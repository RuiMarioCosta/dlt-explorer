use memchr::memmem::Finder;

use crate::dlt::error::{ParseError, ParseErrorKind};
use crate::dlt::protocol::*;

/// v1 standard header minimum size: HTYP(1) + MCNT(1) + LEN(2) = 4
const V1_STD_HEADER_MIN: usize = 4;

/// A located DLT v1 frame within memory-mapped data.
pub struct Frame {
    /// Nanosecond-resolution timestamp from the storage header.
    pub storage_timestamp_ns: u64,
    /// Byte offset of the standard header start within the data slice.
    pub msg_start: usize,
    /// Message length from the standard header LEN field.
    pub msg_len: usize,
}

/// Output of v1 frame scanning.
pub struct ScanOutput {
    pub frames: Vec<Frame>,
    pub errors: Vec<ParseError>,
    /// Default storage header ECU for this file.
    pub default_storage_ecu: Option<[u8; 4]>,
    /// Sparse per-frame storage ECU overrides: (frame_index, ecu).
    pub storage_ecu_overrides: Vec<(usize, [u8; 4])>,
}

/// Scan `data` for DLT v1 frames, returning one `Frame` per valid v1 message
/// found, plus any `ParseError`s for malformed frames encountered along the way.
///
/// Frame boundaries are determined by the LEN field in the standard header,
/// not by scanning for the next `DLT\x01` marker. This means false `DLT\x01`
/// sequences embedded in payload data do not cause mis-framing.
///
/// On error the scanner resyncs to the next `DLT\x01` marker.
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

        // Need at least the storage header + v1 standard header minimum
        if storage_end + V1_STD_HEADER_MIN > data.len() {
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

        // Standard header starts immediately after the storage header
        let msg_start = storage_end;
        let htyp = data[msg_start];

        // Version is in bits 5-7 of HTYP
        let version = (htyp >> 5) & 0x07;
        if version != 1 {
            errors.push(ParseError {
                file_index,
                byte_offset: pos as u64,
                kind: ParseErrorKind::InvalidVersion { found: version },
            });
            search_start = pos + 1;
            continue;
        }

        // LEN at bytes 2-3 of the standard header (big-endian)
        let len = u16::from_be_bytes(data[msg_start + 2..msg_start + 4].try_into().unwrap());

        if (len as usize) < V1_STD_HEADER_MIN {
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
                    available: data.len() - msg_start,
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

        // Advance past this message (LEN-driven, skips over payload content)
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

    /// Build a minimal v1 frame (storage header + standard header, no ext header or payload).
    fn minimal_v1_frame() -> Vec<u8> {
        let mut buf = Vec::new();

        // Storage header (16 bytes)
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&100u32.to_le_bytes()); // seconds
        buf.extend_from_slice(&500u32.to_le_bytes()); // microseconds
        buf.extend_from_slice(b"ECU1"); // ECU ID

        // Standard header: HTYP(1) + MCNT(1) + LEN(2) = 4 bytes
        let htyp: u8 = 1 << 5; // version = 1, no optional fields
        buf.push(htyp);
        buf.push(0); // MCNT
        let len: u16 = 4; // just the standard header
        buf.extend_from_slice(&len.to_be_bytes());

        buf
    }

    #[test]
    fn finds_single_v1_frame() {
        let data = minimal_v1_frame();
        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 0);
        assert_eq!(
            out.frames[0].storage_timestamp_ns,
            100 * 1_000_000_000 + 500 * 1_000
        );
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
        assert!(out.storage_ecu_overrides.is_empty());
        assert_eq!(out.frames[0].msg_start, STORAGE_HEADER_SIZE);
        assert_eq!(out.frames[0].msg_len, 4);
    }

    #[test]
    fn finds_multiple_frames() {
        let mut data = minimal_v1_frame();
        data.extend_from_slice(&minimal_v1_frame());
        data.extend_from_slice(&minimal_v1_frame());

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 3);
        assert_eq!(out.errors.len(), 0);
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
        assert!(out.storage_ecu_overrides.is_empty());
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
        // DLT\x01 found but not enough data for storage + standard header
        let out = scan_frames(b"DLT\x01too_short", 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn truncated_mid_header() {
        // Full storage header but standard header cut short
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");
        // Only 3 bytes of standard header instead of 4
        buf.extend_from_slice(&[0x20, 0x00, 0x00]);

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn wrong_version_resyncs() {
        let mut buf = Vec::new();

        // First frame: version = 2 (wrong for v1 scanner)
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");
        let htyp_v2: u8 = 2 << 5;
        buf.push(htyp_v2);
        buf.push(0);
        buf.extend_from_slice(&4u16.to_be_bytes());

        // Second frame: valid v1
        buf.extend_from_slice(&minimal_v1_frame());

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::InvalidVersion { found: 2 });
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
    }

    #[test]
    fn length_mismatch_declared_exceeds_available() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");
        let htyp: u8 = 1 << 5;
        buf.push(htyp);
        buf.push(0);
        // Declared length far exceeds available data
        buf.extend_from_slice(&500u16.to_be_bytes());

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 0);
        assert_eq!(out.errors.len(), 1);
        assert!(matches!(
            out.errors[0].kind,
            ParseErrorKind::LengthMismatch { declared: 500, .. }
        ));
    }

    #[test]
    fn false_marker_in_payload_not_confused_as_frame() {
        // Build a valid v1 frame whose payload contains DLT\x01
        let mut buf = Vec::new();

        // Storage header
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&100u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"ECU1");

        // Standard header: version=1, no optional fields
        let htyp: u8 = 1 << 5;
        buf.push(htyp);
        buf.push(0); // MCNT
        // payload = DLT\x01 (4 bytes), total LEN = 4 (std hdr) + 4 (payload) = 8
        let len: u16 = 8;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(b"DLT\x01"); // false marker in payload

        let out = scan_frames(&buf, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 0);
        assert_eq!(out.frames[0].msg_len, 8);
    }

    #[test]
    fn file_ending_mid_message_after_valid() {
        let mut data = minimal_v1_frame();
        // Append a truncated frame
        data.extend_from_slice(b"DLT\x01");
        data.extend_from_slice(&[0u8; 5]); // not enough for storage + standard header

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 1);
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn mixed_storage_ecu_uses_sparse_overrides() {
        let mut data = minimal_v1_frame();

        let mut second = minimal_v1_frame();
        second[12..16].copy_from_slice(b"ECU2");
        data.extend_from_slice(&second);

        data.extend_from_slice(&minimal_v1_frame());

        let out = scan_frames(&data, 0);
        assert_eq!(out.frames.len(), 3);
        assert_eq!(out.default_storage_ecu, Some(*b"ECU1"));
        assert_eq!(out.storage_ecu_overrides.len(), 1);
        assert_eq!(out.storage_ecu_overrides[0], (1, *b"ECU2"));
    }
}
