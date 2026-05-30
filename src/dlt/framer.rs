use memchr::memmem::Finder;

use super::protocol::*;

/// A located DLT v2 frame within memory-mapped data.
pub struct Frame {
    /// Nanosecond-resolution timestamp from the storage header.
    pub storage_timestamp_ns: u64,
    /// ECU ID from the storage header (4 bytes, null-padded).
    pub storage_ecu: [u8; 4],
    /// Byte offset of the base header start within the data slice.
    pub msg_start: usize,
    /// Message length from the base header LEN field.
    pub msg_len: usize,
}

/// Scan `data` for DLT v2 frames, returning one `Frame` per valid v2 message found.
///
/// Non-v2 messages (e.g. v1) are silently skipped.
pub fn scan_frames(data: &[u8]) -> Vec<Frame> {
    let finder = Finder::new(DLT_STORAGE_HEADER_PATTERN);
    let mut frames = Vec::new();
    let mut search_start = 0;

    while let Some(rel_pos) = finder.find(&data[search_start..]) {
        let pos = search_start + rel_pos;
        let storage_end = pos + STORAGE_HEADER_SIZE;

        // Need at least the storage header + minimum base header to proceed
        if storage_end + BASE_HEADER_MIN_SIZE > data.len() {
            break;
        }

        // Parse storage header TMSP2 (LE)
        let seconds = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());
        let nanoseconds = u32::from_le_bytes(data[pos + 8..pos + 12].try_into().unwrap());
        let mut ecu = [0u8; 4];
        ecu.copy_from_slice(&data[pos + 13..pos + 17]);

        let timestamp_ns = seconds as u64 * 1_000_000_000 + nanoseconds as u64;

        // Base header starts immediately after the storage header
        let msg_start = storage_end;
        let htyp2 = u32::from_be_bytes(data[msg_start..msg_start + 4].try_into().unwrap());

        // Skip non-v2 messages
        if htyp2_version(htyp2) != PROTOCOL_VERSION_2 {
            search_start = pos + 1;
            continue;
        }

        let len =
            u16::from_be_bytes(data[msg_start + 5..msg_start + 7].try_into().unwrap()) as usize;

        if len < BASE_HEADER_MIN_SIZE || msg_start + len > data.len() {
            search_start = pos + 1;
            continue;
        }

        frames.push(Frame {
            storage_timestamp_ns: timestamp_ns,
            storage_ecu: ecu,
            msg_start,
            msg_len: len,
        });

        // Advance past this message
        search_start = msg_start + len;
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal v2 frame (storage header + base header only, no payload).
    fn minimal_v2_frame() -> Vec<u8> {
        let mut buf = Vec::new();

        // Storage header
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&100u32.to_le_bytes()); // seconds
        buf.extend_from_slice(&500u32.to_le_bytes()); // nanoseconds
        buf.push(0); // flags
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
        let frames = scan_frames(&data);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].storage_timestamp_ns, 100 * 1_000_000_000 + 500);
        assert_eq!(&frames[0].storage_ecu, b"ECU1");
    }

    #[test]
    fn skips_v1_frame() {
        let mut buf = Vec::new();
        // Storage header (use v2 format for framing, but v1 base header)
        buf.extend_from_slice(b"DLT\x01");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.push(0);
        buf.extend_from_slice(b"ECU1");

        // Base header with VERS=1
        let htyp2 = build_htyp2(CNTI_VERBOSE, false, false, false, 1);
        buf.extend_from_slice(&htyp2.to_be_bytes());
        buf.push(0); // MCNT
        buf.extend_from_slice(&9u16.to_be_bytes());
        buf.push(0);
        buf.push(0);

        let frames = scan_frames(&buf);
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn empty_input() {
        let frames = scan_frames(&[]);
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn truncated_storage_header() {
        let frames = scan_frames(b"DLT\x01too_short");
        assert_eq!(frames.len(), 0);
    }
}
