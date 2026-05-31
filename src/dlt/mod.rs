mod dlt_common;
mod dlt_protocol;
pub mod v1;

pub mod error;
pub mod framer;
pub mod header;
pub mod intern;
pub mod payload;
pub mod protocol;

use anyhow::Result;
use memmap2::Mmap;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use error::{ParseError, ParseErrorKind};
use framer::scan_frames;
use header::parse_v2_header;
use intern::InternTable;

/// DLT v2 parsed data in columnar (struct-of-arrays) layout.
///
/// Payloads are stored lazily as mmap-backed byte ranges, decoded on demand.
pub struct Dlt {
    mmaps: Vec<Mmap>,
    intern: InternTable,
    apid: Vec<u16>,
    ctid: Vec<u16>,
    ecu: Vec<u16>,
    session_id: Vec<u32>,
    storage_timestamp_ns: Vec<u64>,
    message_timestamp_ns: Vec<u64>,
    message_type: Vec<u8>,
    log_level: Vec<u8>,
    payload_loc: Vec<(u16, u32, u32)>, // (mmap_index, offset, len)
}

impl Dlt {
    /// Open and parse one or more DLT v2 files.
    ///
    /// Returns successfully parsed messages alongside any errors encountered.
    /// Non-v2 messages and malformed frames are recorded as errors and skipped.
    pub fn open(paths: Vec<PathBuf>) -> Result<(Self, Vec<ParseError>)> {
        let mut mmaps = Vec::with_capacity(paths.len());
        let mut intern = InternTable::new();
        let mut apid = Vec::new();
        let mut ctid = Vec::new();
        let mut ecu = Vec::new();
        let mut session_id = Vec::new();
        let mut storage_timestamp_ns = Vec::new();
        let mut message_timestamp_ns = Vec::new();
        let mut message_type = Vec::new();
        let mut log_level = Vec::new();
        let mut payload_loc = Vec::new();
        let mut all_errors = Vec::new();

        for (file_idx, path) in paths.iter().enumerate() {
            let file = File::open(path)?;
            // SAFETY: the file is read-only and the Mmap is kept alive for the
            // lifetime of the Dlt struct.
            let mmap = unsafe { Mmap::map(&file)? };

            let (frames, frame_errors) = scan_frames(&mmap, file_idx as u16);
            all_errors.extend(frame_errors);

            for frame in frames {
                let msg = &mmap[frame.msg_start..frame.msg_start + frame.msg_len];

                let Some(hdr) = parse_v2_header(msg) else {
                    all_errors.push(ParseError {
                        file_index: file_idx as u16,
                        byte_offset: frame.msg_start as u64,
                        kind: ParseErrorKind::InvalidExtensionField,
                    });
                    continue;
                };

                let apid_str = match &hdr.apid {
                    Some(b) => std::str::from_utf8(b).unwrap_or(""),
                    None => "",
                };
                let ctid_str = match &hdr.ctid {
                    Some(b) => std::str::from_utf8(b).unwrap_or(""),
                    None => "",
                };
                let ecu_str = match &hdr.ecu {
                    Some(b) => std::str::from_utf8(b).unwrap_or(""),
                    None => "",
                };

                apid.push(intern.insert(apid_str));
                ctid.push(intern.insert(ctid_str));
                ecu.push(intern.insert(ecu_str));
                session_id.push(hdr.session_id.unwrap_or(0));
                storage_timestamp_ns.push(frame.storage_timestamp_ns);
                message_timestamp_ns.push(hdr.message_timestamp_ns);
                message_type.push(hdr.message_type);
                log_level.push(hdr.log_level);

                let payload_offset_in_mmap = frame.msg_start + hdr.payload_offset;
                payload_loc.push((
                    file_idx as u16,
                    payload_offset_in_mmap as u32,
                    hdr.payload_len as u32,
                ));
            }

            mmaps.push(mmap);
        }

        Ok((Dlt {
            mmaps,
            intern,
            apid,
            ctid,
            ecu,
            session_id,
            storage_timestamp_ns,
            message_timestamp_ns,
            message_type,
            log_level,
            payload_loc,
        }, all_errors))
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

    pub fn storage_timestamp_ns(&self, row: usize) -> u64 {
        self.storage_timestamp_ns[row]
    }

    pub fn ecu(&self, row: usize) -> &str {
        self.intern.resolve(self.ecu[row])
    }

    pub fn session_id(&self, row: usize) -> u32 {
        self.session_id[row]
    }

    pub fn message_timestamp_ns(&self, row: usize) -> u64 {
        self.message_timestamp_ns[row]
    }

    pub fn log_level(&self, row: usize) -> u8 {
        self.log_level[row]
    }

    pub fn message_type(&self, row: usize) -> u8 {
        self.message_type[row]
    }

    pub fn payload_raw(&self, row: usize) -> &[u8] {
        let (mmap_idx, offset, len) = self.payload_loc[row];
        &self.mmaps[mmap_idx as usize][offset as usize..(offset + len) as usize]
    }
}

impl fmt::Debug for Dlt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dlt")
            .field("messages", &self.len())
            .field("files", &self.mmaps.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// V2MessageBuilder — test helper for constructing synthetic v2 messages
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod test_helpers {
    use super::protocol::*;

    /// Builds a spec-compliant AUTOSAR PRS v2 DLT message as raw bytes.
    ///
    /// Produces: storage header + base header + extension header + payload.
    pub struct V2MessageBuilder {
        storage_seconds: u32,
        storage_nanoseconds: u32,
        storage_ecu: [u8; 4],
        apid: Option<[u8; 4]>,
        ctid: Option<[u8; 4]>,
        ecu: Option<[u8; 4]>,
        session_id: Option<u32>,
        message_timestamp_ns: Option<u64>,
        privacy_level: Option<u8>,
        message_type: u8,
        log_level: u8,
        verbose_payload: Vec<u8>,
        noar: u8,
    }

    impl V2MessageBuilder {
        pub fn new() -> Self {
            Self {
                storage_seconds: 1000,
                storage_nanoseconds: 500_000,
                storage_ecu: *b"ECU1",
                apid: None,
                ctid: None,
                ecu: None,
                session_id: None,
                message_timestamp_ns: None,
                privacy_level: None,
                message_type: MESSAGE_TYPE_LOG,
                log_level: LOG_LEVEL_INFO,
                verbose_payload: Vec::new(),
                noar: 0,
            }
        }

        pub fn with_storage_ecu(mut self, ecu: &[u8; 4]) -> Self {
            self.storage_ecu = *ecu;
            self
        }

        pub fn with_storage_timestamp(mut self, seconds: u32, nanoseconds: u32) -> Self {
            self.storage_seconds = seconds;
            self.storage_nanoseconds = nanoseconds;
            self
        }

        pub fn with_apid(mut self, apid: &str) -> Self {
            let mut buf = [0u8; 4];
            let bytes = apid.as_bytes();
            let n = bytes.len().min(4);
            buf[..n].copy_from_slice(&bytes[..n]);
            self.apid = Some(buf);
            self
        }

        pub fn with_ctid(mut self, ctid: &str) -> Self {
            let mut buf = [0u8; 4];
            let bytes = ctid.as_bytes();
            let n = bytes.len().min(4);
            buf[..n].copy_from_slice(&bytes[..n]);
            self.ctid = Some(buf);
            self
        }

        pub fn without_apid(mut self) -> Self {
            self.apid = None;
            self
        }

        pub fn without_ctid(mut self) -> Self {
            self.ctid = None;
            self
        }

        pub fn with_ecu(mut self, ecu: &str) -> Self {
            let mut buf = [0u8; 4];
            let bytes = ecu.as_bytes();
            let n = bytes.len().min(4);
            buf[..n].copy_from_slice(&bytes[..n]);
            self.ecu = Some(buf);
            self
        }

        pub fn with_session_id(mut self, id: u32) -> Self {
            self.session_id = Some(id);
            self
        }

        pub fn with_timestamp_ns(mut self, ns: u64) -> Self {
            self.message_timestamp_ns = Some(ns);
            self
        }

        pub fn with_privacy_level(mut self, level: u8) -> Self {
            self.privacy_level = Some(level);
            self
        }

        pub fn with_message_type(mut self, mstp: u8) -> Self {
            self.message_type = mstp;
            self
        }

        pub fn with_log_level(mut self, mtin: u8) -> Self {
            self.log_level = mtin;
            self
        }

        /// Add a verbose UTF-8 string argument to the payload.
        pub fn with_verbose_string(mut self, s: &str) -> Self {
            // TypeInfo: STRG | SCOD_UTF8 = 0x00008200
            let type_info: u32 = 0x0000_8200;
            self.verbose_payload
                .extend_from_slice(&type_info.to_be_bytes());
            let bytes = s.as_bytes();
            let len = (bytes.len() + 1) as u16; // +1 for null terminator
            self.verbose_payload
                .extend_from_slice(&len.to_be_bytes());
            self.verbose_payload.extend_from_slice(bytes);
            self.verbose_payload.push(0); // null terminator
            self.noar += 1;
            self
        }

        /// Build the complete message as a byte vector.
        pub fn build(self) -> Vec<u8> {
            let mut msg = Vec::new();

            // --- Storage header (17 bytes) ---
            msg.extend_from_slice(b"DLT\x01");
            msg.extend_from_slice(&self.storage_seconds.to_le_bytes());
            msg.extend_from_slice(&self.storage_nanoseconds.to_le_bytes());
            msg.push(0x00); // flags
            msg.extend_from_slice(&self.storage_ecu);

            // --- Build HTYP2 ---
            let has_wacid = self.apid.is_some() || self.ctid.is_some();
            let has_weid = self.ecu.is_some();
            let has_wsid = self.session_id.is_some();
            let has_wpvl = self.privacy_level.is_some();

            let htyp2 = build_htyp2_full(
                CNTI_VERBOSE,
                has_weid,
                has_wacid,
                has_wsid,
                PROTOCOL_VERSION_2,
                false, // WSFLN
                false, // WTGS
                has_wpvl,
                false, // WSGM
            );

            // --- Compute message length ---
            // Base header: HTYP2(4) + MCNT(1) + LEN(2) = 7
            // + MSIN(1) + NOAR(1) = 2 (verbose)
            // + TMSP2(9) (data message)
            let base_size = 7 + 2 + 9;
            let ext_size = {
                let mut sz = 0;
                if has_weid {
                    sz += 1 + 4; // length byte + 4 value bytes
                }
                if has_wacid {
                    sz += self.apid.map_or(1, |_| 5); // 1 len byte + 4 value (or just len=0)
                    sz += self.ctid.map_or(1, |_| 5);
                }
                if has_wsid {
                    sz += 4; // fixed 4 bytes
                }
                if has_wpvl {
                    sz += 1; // 1 byte
                }
                sz
            };
            let payload_size = self.verbose_payload.len();
            let total_len = base_size + ext_size + payload_size;

            // --- Base header ---
            msg.extend_from_slice(&htyp2.to_be_bytes());
            msg.push(0); // MCNT
            msg.extend_from_slice(&(total_len as u16).to_be_bytes());

            // MSIN
            msg.push(build_msin(self.message_type, self.log_level));
            msg.push(self.noar); // NOAR

            // TMSP2
            let ts_ns = self.message_timestamp_ns.unwrap_or(0);
            msg.extend_from_slice(&encode_tmsp2(ts_ns));

            // --- Extension header ---
            if has_weid {
                msg.push(4); // length
                msg.extend_from_slice(&self.ecu.unwrap());
            }

            if has_wacid {
                if let Some(apid) = self.apid {
                    msg.push(4);
                    msg.extend_from_slice(&apid);
                } else {
                    msg.push(0);
                }

                if let Some(ctid) = self.ctid {
                    msg.push(4);
                    msg.extend_from_slice(&ctid);
                } else {
                    msg.push(0);
                }
            }

            if let Some(sid) = self.session_id {
                msg.extend_from_slice(&sid.to_be_bytes());
            }

            if let Some(pvl) = self.privacy_level {
                msg.push(pvl);
            }

            // --- Payload ---
            msg.extend_from_slice(&self.verbose_payload);

            msg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::V2MessageBuilder;
    use super::*;
    use error::ParseErrorKind;
    use std::io::Write;

    #[test]
    fn v2_roundtrip_single_message() {
        let ts_ns = 1000u64 * 1_000_000_000 + 42;
        let msg_bytes = V2MessageBuilder::new()
            .with_apid("APP1")
            .with_ctid("CTX1")
            .with_storage_timestamp(1000, 500_000)
            .with_timestamp_ns(ts_ns)
            .with_verbose_string("hello")
            .build();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&msg_bytes).unwrap();
        }

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();

        assert_eq!(errors.len(), 0);
        assert_eq!(dlt.len(), 1);
        assert!(!dlt.is_empty());
        assert_eq!(dlt.apid(0), "APP1");
        assert_eq!(dlt.ctid(0), "CTX1");
        assert_eq!(
            dlt.storage_timestamp_ns(0),
            1000 * 1_000_000_000 + 500_000
        );
        assert_eq!(dlt.message_timestamp_ns(0), ts_ns);
        assert_eq!(dlt.message_type(0), protocol::MESSAGE_TYPE_LOG);
        assert_eq!(dlt.log_level(0), protocol::LOG_LEVEL_INFO);

        // Payload should contain the verbose string argument bytes
        let raw = dlt.payload_raw(0);
        assert!(!raw.is_empty());
    }

    #[test]
    fn v2_all_fields_populated() {
        let ts_ns = 42u64 * 1_000_000_000 + 999_999_999;
        let msg_bytes = V2MessageBuilder::new()
            .with_apid("AP01")
            .with_ctid("CT01")
            .with_ecu("ECU2")
            .with_session_id(0xDEAD)
            .with_timestamp_ns(ts_ns)
            .with_privacy_level(5)
            .with_message_type(protocol::MESSAGE_TYPE_TRACE)
            .with_log_level(protocol::LOG_LEVEL_WARN)
            .with_verbose_string("world")
            .build();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("all_fields.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&msg_bytes).unwrap();
        }

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();

        assert_eq!(errors.len(), 0);
        assert_eq!(dlt.len(), 1);
        assert_eq!(dlt.apid(0), "AP01");
        assert_eq!(dlt.ctid(0), "CT01");
        assert_eq!(dlt.ecu(0), "ECU2");
        assert_eq!(dlt.session_id(0), 0xDEAD);
        assert_eq!(dlt.message_timestamp_ns(0), ts_ns);
        assert_eq!(dlt.message_type(0), protocol::MESSAGE_TYPE_TRACE);
        assert_eq!(dlt.log_level(0), protocol::LOG_LEVEL_WARN);

        let raw = dlt.payload_raw(0);
        assert!(!raw.is_empty());
    }

    #[test]
    fn v2_log_level_and_message_type_variants() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("variants.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            for (mstp, mtin) in [
                (protocol::MESSAGE_TYPE_LOG, protocol::LOG_LEVEL_FATAL),
                (protocol::MESSAGE_TYPE_TRACE, protocol::LOG_LEVEL_ERROR),
                (protocol::MESSAGE_TYPE_NETWORK, protocol::LOG_LEVEL_WARN),
                (protocol::MESSAGE_TYPE_CONTROL, protocol::LOG_LEVEL_INFO),
            ] {
                let msg = V2MessageBuilder::new()
                    .with_apid("APP1")
                    .with_ctid("CTX1")
                    .with_message_type(mstp)
                    .with_log_level(mtin)
                    .build();
                f.write_all(&msg).unwrap();
            }
        }

        let (dlt, _errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 4);

        assert_eq!(dlt.message_type(0), protocol::MESSAGE_TYPE_LOG);
        assert_eq!(dlt.log_level(0), protocol::LOG_LEVEL_FATAL);
        assert_eq!(dlt.message_type(1), protocol::MESSAGE_TYPE_TRACE);
        assert_eq!(dlt.log_level(1), protocol::LOG_LEVEL_ERROR);
        assert_eq!(dlt.message_type(2), protocol::MESSAGE_TYPE_NETWORK);
        assert_eq!(dlt.log_level(2), protocol::LOG_LEVEL_WARN);
        assert_eq!(dlt.message_type(3), protocol::MESSAGE_TYPE_CONTROL);
        assert_eq!(dlt.log_level(3), protocol::LOG_LEVEL_INFO);
    }

    #[test]
    fn v2_flags_cleared_returns_empty_sentinels() {
        let msg_bytes = V2MessageBuilder::new().build();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_ext.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&msg_bytes).unwrap();
        }

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(errors.len(), 0);
        assert_eq!(dlt.len(), 1);
        assert_eq!(dlt.apid(0), "");
        assert_eq!(dlt.ctid(0), "");
        assert_eq!(dlt.ecu(0), "");
        assert_eq!(dlt.session_id(0), 0);
    }

    #[test]
    fn v2_open_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.dlt");
        std::fs::File::create(&path).unwrap();

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 0);
        assert_eq!(errors.len(), 0);
        assert!(dlt.is_empty());
    }

    #[test]
    fn v2_open_v1_file_returns_zero_messages() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/testfile_control_messages.dlt");

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        // v1 messages reported as InvalidVersion errors
        assert_eq!(dlt.len(), 0);
        assert!(!errors.is_empty());
        assert!(errors.iter().all(|e| matches!(e.kind, ParseErrorKind::InvalidVersion { .. })));
    }

    #[test]
    fn v2_empty_file_no_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.dlt");
        std::fs::File::create(&path).unwrap();

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn v2_truncated_file_collects_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("truncated.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            // Write a valid message followed by a truncated storage header
            let msg = V2MessageBuilder::new()
                .with_apid("APP1")
                .with_ctid("CTX1")
                .build();
            f.write_all(&msg).unwrap();
            // Truncated: DLT\x01 + a few bytes, not enough for storage + base header
            f.write_all(b"DLT\x01\x00\x00").unwrap();
        }

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ParseErrorKind::Truncated);
    }

    #[test]
    fn v2_corrupted_message_in_middle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupted_middle.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            // Message 1 (valid)
            f.write_all(&V2MessageBuilder::new().with_apid("APP1").with_ctid("CTX1").build()).unwrap();
            // Corrupted message: valid storage header, but version=3
            let mut bad = Vec::new();
            bad.extend_from_slice(b"DLT\x01");
            bad.extend_from_slice(&0u32.to_le_bytes());
            bad.extend_from_slice(&0u32.to_le_bytes());
            bad.push(0);
            bad.extend_from_slice(b"ECU1");
            let bad_htyp2 = protocol::build_htyp2(protocol::CNTI_VERBOSE, false, false, false, 3);
            bad.extend_from_slice(&bad_htyp2.to_be_bytes());
            bad.push(0);
            bad.extend_from_slice(&9u16.to_be_bytes());
            bad.push(0);
            bad.push(0);
            f.write_all(&bad).unwrap();
            // Message 2 (valid)
            f.write_all(&V2MessageBuilder::new().with_apid("APP2").with_ctid("CTX2").build()).unwrap();
            // Message 3 (valid)
            f.write_all(&V2MessageBuilder::new().with_apid("APP3").with_ctid("CTX3").build()).unwrap();
        }

        let (dlt, errors) = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 3);
        assert_eq!(errors.len(), 1);
    }
}
