mod framer;
mod header;
mod payload;
mod protocol;

use anyhow::Result;
use memmap2::Mmap;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use crate::dlt::error::{ParseError, ParseErrorKind};
use crate::dlt::intern::InternTable;
use framer::scan_frames;
use header::parse_v1_header;
use protocol::{msin_mstp, msin_mtin};

/// DLT v1 parsed data in columnar (struct-of-arrays) layout.
///
/// Payloads are stored lazily as mmap-backed byte ranges, decoded on demand.
pub struct Dlt {
    mmaps: Vec<Mmap>,
    intern: InternTable,
    htyp: Vec<u8>,
    msin: Vec<u8>,
    storage_timestamp_ns: Vec<u64>,
    message_timestamp_ns: Vec<u64>,
    ecu: Vec<u16>,
    apid: Vec<u16>,
    ctid: Vec<u16>,
    session_id: Vec<u32>,
    payload_loc: Vec<(u16, u32, u32)>, // (mmap_index, offset, len)
}

impl Dlt {
    /// Open and parse one or more DLT v1 files.
    ///
    /// Returns successfully parsed messages alongside any errors encountered.
    /// Non-v1 messages and malformed frames are recorded as errors and skipped.
    pub fn open(paths: Vec<PathBuf>) -> Result<(Self, Vec<ParseError>)> {
        let mut mmaps = Vec::with_capacity(paths.len());
        let mut intern = InternTable::new();
        let mut htyp = Vec::new();
        let mut msin = Vec::new();
        let mut storage_timestamp_ns = Vec::new();
        let mut message_timestamp_ns = Vec::new();
        let mut ecu = Vec::new();
        let mut apid = Vec::new();
        let mut ctid = Vec::new();
        let mut session_id = Vec::new();
        let mut payload_loc = Vec::new();
        let mut all_errors = Vec::new();

        for (file_idx, path) in paths.iter().enumerate() {
            let file = File::open(path)?;
            // SAFETY: the file is read-only and the Mmap is kept alive for the
            // lifetime of the Dlt struct.
            let mmap = unsafe { Mmap::map(&file)? };

            let scan = scan_frames(&mmap, file_idx as u16);
            all_errors.extend(scan.errors);

            let mut next_override = 0usize;
            for (frame_idx, frame) in scan.frames.into_iter().enumerate() {
                let msg = &mmap[frame.msg_start..frame.msg_start + frame.msg_len];

                // Resolve this frame's storage ECU first so override cursor stays
                // aligned even when message ECU is present and takes precedence.
                let mut storage_ecu = scan.default_storage_ecu;
                if next_override < scan.storage_ecu_overrides.len()
                    && scan.storage_ecu_overrides[next_override].0 == frame_idx
                {
                    storage_ecu = Some(scan.storage_ecu_overrides[next_override].1);
                    next_override += 1;
                }

                let hdr = match parse_v1_header(msg) {
                    Ok(hdr) => hdr,
                    Err(kind) => {
                        all_errors.push(ParseError {
                            file_index: file_idx as u16,
                            byte_offset: frame.msg_start as u64,
                            kind,
                        });
                        continue;
                    }
                };

                let ecu_id = match &hdr.ecu {
                    Some(b) => {
                        let ecu_str = std::str::from_utf8(b).unwrap_or("");
                        intern.insert(ecu_str.trim_end_matches('\0'))
                    }
                    None => {
                        let Some(storage_ecu) = storage_ecu else {
                            all_errors.push(ParseError {
                                file_index: file_idx as u16,
                                byte_offset: frame.msg_start as u64,
                                kind: ParseErrorKind::InvalidStandardHeader,
                            });
                            continue;
                        };

                        let ecu_str = std::str::from_utf8(&storage_ecu).unwrap_or("");
                        intern.insert(ecu_str.trim_end_matches('\0'))
                    }
                };
                let apid_id = match &hdr.apid {
                    Some(b) => {
                        let apid_str = std::str::from_utf8(b).unwrap_or("");
                        intern.insert(apid_str.trim_end_matches('\0'))
                    }
                    None => intern.insert(""),
                };
                let ctid_id = match &hdr.ctid {
                    Some(b) => {
                        let ctid_str = std::str::from_utf8(b).unwrap_or("");
                        intern.insert(ctid_str.trim_end_matches('\0'))
                    }
                    None => intern.insert(""),
                };

                htyp.push(hdr.htyp);
                msin.push(hdr.msin);
                ecu.push(ecu_id);
                apid.push(apid_id);
                ctid.push(ctid_id);
                session_id.push(hdr.session_id.unwrap_or(0));
                storage_timestamp_ns.push(frame.storage_timestamp_ns);
                message_timestamp_ns.push(hdr.message_timestamp_ns);

                let payload_offset_in_mmap = frame.msg_start + hdr.payload_offset;
                payload_loc.push((
                    file_idx as u16,
                    payload_offset_in_mmap as u32,
                    hdr.payload_len as u32,
                ));
            }

            mmaps.push(mmap);
        }

        Ok((
            Dlt {
                mmaps,
                intern,
                htyp,
                msin,
                storage_timestamp_ns,
                message_timestamp_ns,
                ecu,
                apid,
                ctid,
                session_id,
                payload_loc,
            },
            all_errors,
        ))
    }

    pub fn len(&self) -> usize {
        self.apid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.apid.is_empty()
    }

    pub fn apid(&self, row: usize) -> &str {
        self.intern.resolve(self.apid[row])
    }

    pub fn ctid(&self, row: usize) -> &str {
        self.intern.resolve(self.ctid[row])
    }

    pub fn ecu(&self, row: usize) -> &str {
        self.intern.resolve(self.ecu[row])
    }

    pub fn storage_timestamp_ns(&self, row: usize) -> u64 {
        self.storage_timestamp_ns[row]
    }

    pub fn message_timestamp_ns(&self, row: usize) -> u64 {
        self.message_timestamp_ns[row]
    }

    pub fn message_type(&self, row: usize) -> u8 {
        msin_mstp(self.msin[row])
    }

    pub fn message_type_info(&self, row: usize) -> u8 {
        msin_mtin(self.msin[row])
    }

    pub fn session_id(&self, row: usize) -> u32 {
        self.session_id[row]
    }

    pub fn payload_raw(&self, row: usize) -> &[u8] {
        let (mmap_idx, offset, len) = self.payload_loc[row];
        &self.mmaps[mmap_idx as usize][offset as usize..(offset + len) as usize]
    }

    pub fn payload_text(&self, row: usize) -> String {
        let raw = self.payload_raw(row);
        let htyp = self.htyp[row];
        let msin = self.msin[row];
        payload::decode_payload(htyp, msin, raw)
    }

    /// Sorted, deduplicated list of all APID strings seen.
    pub fn unique_apids(&self) -> Vec<&str> {
        unique_interned(&self.apid, &self.intern)
    }

    /// Sorted, deduplicated list of all CTID strings seen.
    pub fn unique_ctids(&self) -> Vec<&str> {
        unique_interned(&self.ctid, &self.intern)
    }

    /// Sorted, deduplicated list of all ECU strings seen.
    pub fn unique_ecus(&self) -> Vec<&str> {
        unique_interned(&self.ecu, &self.intern)
    }
}

/// Collect sorted unique non-empty strings from an interned column.
fn unique_interned<'a>(col: &[u16], intern: &'a InternTable) -> Vec<&'a str> {
    let mut ids: Vec<u16> = col.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut result: Vec<&str> = ids
        .into_iter()
        .map(|id| intern.resolve(id))
        .filter(|s| !s.is_empty())
        .collect();
    result.sort_unstable();
    result
}

impl fmt::Debug for Dlt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dlt")
            .field("messages", &self.len())
            .field("files", &self.mmaps.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn test_data_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data");
        path.push(filename);
        path
    }

    #[test]
    fn open_single_file_parses_messages() {
        let path = test_data_path("testfile_single_payloads.dlt");
        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert!(!dlt.is_empty());
        // Errors may occur for per-message issues, but open should succeed
        let _ = errors;
    }

    #[test]
    fn len_and_is_empty() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        assert!(!dlt.is_empty());
    }

    #[test]
    fn row_accessors_return_values() {
        let path = test_data_path("testfile_single_payloads.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        assert!(!dlt.is_empty());
        // Verify accessors don't panic for valid rows
        for row in 0..dlt.len() {
            let _ = dlt.apid(row);
            let _ = dlt.ctid(row);
            let _ = dlt.ecu(row);
            let _ = dlt.storage_timestamp_ns(row);
            let _ = dlt.message_timestamp_ns(row);
            let _ = dlt.message_type(row);
            let _ = dlt.message_type_info(row);
            let _ = dlt.session_id(row);
            let _ = dlt.payload_raw(row);
            let _ = dlt.payload_text(row);
        }
    }

    #[test]
    fn apid_ctid_ecu_resolved() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        // All messages in this file have APID="APP" and CTID="CON"
        for row in 0..dlt.len() {
            assert_eq!(dlt.apid(row), "APP");
            assert_eq!(dlt.ctid(row), "CON");
        }
    }

    #[test]
    fn unique_apids_sorted_deduped() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        let apids = dlt.unique_apids();
        assert!(apids.contains(&"APP"));
        // Verify sorted
        for w in apids.windows(2) {
            assert!(w[0] <= w[1]);
        }
    }

    #[test]
    fn unique_ctids_sorted_deduped() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        let ctids = dlt.unique_ctids();
        assert!(ctids.contains(&"CON"));
    }

    #[test]
    fn unique_ecus() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        let ecus = dlt.unique_ecus();
        assert!(!ecus.is_empty());
    }

    #[test]
    fn debug_shows_counts() {
        let path = test_data_path("testfile_control_messages.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        let dbg = format!("{:?}", dlt);
        assert!(dbg.contains("messages"));
        assert!(dbg.contains("files"));
    }

    #[test]
    fn multi_file_open() {
        let paths = vec![
            test_data_path("testfile_control_messages.dlt"),
            test_data_path("testfile_single_payloads.dlt"),
        ];
        let (dlt, _) = Dlt::open(paths).unwrap();
        // Should have messages from both files
        assert!(!dlt.is_empty());
        // Payloads from second file should be accessible
        for row in 0..dlt.len() {
            let _ = dlt.payload_raw(row);
        }
    }

    #[test]
    fn open_100k_rows() {
        let path = test_data_path("testfile_100k_rows.dlt");
        let (dlt, _) = Dlt::open(vec![path]).unwrap();
        // The file contains 100k rows
        assert_eq!(dlt.len(), 100_000);
    }

    #[test]
    fn falls_back_to_storage_header_ecu_when_message_ecu_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1_storage_ecu_fallback.dlt");
        let mut file = std::fs::File::create(&path).unwrap();

        let mut frame = Vec::new();
        frame.extend_from_slice(b"DLT\x01");
        frame.extend_from_slice(&1u32.to_le_bytes());
        frame.extend_from_slice(&2u32.to_le_bytes());
        frame.extend_from_slice(b"ECU1");

        // HTYP version=1 only (no WEID), MCNT=0, LEN=4.
        frame.push(1 << 5);
        frame.push(0);
        frame.extend_from_slice(&4u16.to_be_bytes());

        file.write_all(&frame).unwrap();
        file.flush().unwrap();

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert!(errors.is_empty());
        assert_eq!(dlt.len(), 1);
        assert_eq!(dlt.ecu(0), "ECU1");
    }

    #[test]
    fn storage_and_message_ecu_combinations_resolve_expected() {
        fn write_v1_frame(
            file: &mut std::fs::File,
            seconds: u32,
            storage_ecu: [u8; 4],
            message_ecu: Option<[u8; 4]>,
        ) {
            let mut frame = Vec::new();
            frame.extend_from_slice(b"DLT\x01");
            frame.extend_from_slice(&seconds.to_le_bytes());
            frame.extend_from_slice(&0u32.to_le_bytes());
            frame.extend_from_slice(&storage_ecu);

            let mut htyp = 1 << 5; // version = 1
            if message_ecu.is_some() {
                htyp |= 0x04; // WEID present
            }
            frame.push(htyp);
            frame.push(0); // MCNT

            let len: u16 = if message_ecu.is_some() { 8 } else { 4 };
            frame.extend_from_slice(&len.to_be_bytes());
            if let Some(ecu) = message_ecu {
                frame.extend_from_slice(&ecu);
            }

            file.write_all(&frame).unwrap();
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1_ecu_combinations.dlt");
        let mut file = std::fs::File::create(&path).unwrap();

        let cases: [([u8; 4], Option<[u8; 4]>, &str); 7] = [
            (*b"ST01", None, "ST01"),
            (*b"ST02", None, "ST02"),
            (*b"ST03", Some(*b"MS03"), "MS03"),
            (*b"ST01", Some(*b"MS04"), "MS04"),
            (*b"ST02", Some(*b"MS05"), "MS05"),
            (*b"ST03", None, "ST03"),
            (*b"ST03", None, "ST03"),
        ];

        for (idx, (storage_ecu, message_ecu, _)) in cases.iter().enumerate() {
            write_v1_frame(&mut file, idx as u32 + 1, *storage_ecu, *message_ecu);
        }
        file.flush().unwrap();

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert!(errors.is_empty());
        assert_eq!(dlt.len(), cases.len());

        for (row, (_, _, expected)) in cases.iter().enumerate() {
            assert_eq!(dlt.ecu(row), *expected);
        }
    }

    #[test]
    fn nonexistent_file_returns_error() {
        let result = Dlt::open(vec![PathBuf::from("nonexistent.dlt")]);
        assert!(result.is_err());
    }
}
