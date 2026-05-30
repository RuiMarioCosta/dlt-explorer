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
    storage_timestamp_ns: Vec<u64>,
    payload_loc: Vec<(u16, u32, u32)>, // (mmap_index, offset, len)
}

impl Dlt {
    /// Open and parse one or more DLT v2 files.
    ///
    /// Non-v2 messages are silently skipped.
    pub fn open(paths: Vec<PathBuf>) -> Result<Self> {
        let mut mmaps = Vec::with_capacity(paths.len());
        let mut intern = InternTable::new();
        let mut apid = Vec::new();
        let mut ctid = Vec::new();
        let mut storage_timestamp_ns = Vec::new();
        let mut payload_loc = Vec::new();

        for (file_idx, path) in paths.iter().enumerate() {
            let file = File::open(path)?;
            // SAFETY: the file is read-only and the Mmap is kept alive for the
            // lifetime of the Dlt struct.
            let mmap = unsafe { Mmap::map(&file)? };

            for frame in scan_frames(&mmap) {
                let msg = &mmap[frame.msg_start..frame.msg_start + frame.msg_len];

                let Some(hdr) = parse_v2_header(msg) else {
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

                apid.push(intern.insert(apid_str));
                ctid.push(intern.insert(ctid_str));
                storage_timestamp_ns.push(frame.storage_timestamp_ns);

                let payload_offset_in_mmap = frame.msg_start + hdr.payload_offset;
                payload_loc.push((
                    file_idx as u16,
                    payload_offset_in_mmap as u32,
                    hdr.payload_len as u32,
                ));
            }

            mmaps.push(mmap);
        }

        Ok(Dlt {
            mmaps,
            intern,
            apid,
            ctid,
            storage_timestamp_ns,
            payload_loc,
        })
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
            let htyp2 = build_htyp2(CNTI_VERBOSE, false, has_wacid, false, PROTOCOL_VERSION_2);

            // --- Compute message length ---
            // Base header: HTYP2(4) + MCNT(1) + LEN(2) = 7
            // + MSIN(1) + NOAR(1) = 2 (verbose)
            // + TMSP2(9) (data message)
            let base_size = 7 + 2 + 9;
            let ext_size = if has_wacid {
                let apid_size = self.apid.map_or(1, |_| 5); // 1 len byte + 4 value (or just len=0)
                let ctid_size = self.ctid.map_or(1, |_| 5);
                apid_size + ctid_size
            } else {
                0
            };
            let payload_size = self.verbose_payload.len();
            let total_len = base_size + ext_size + payload_size;

            // --- Base header ---
            msg.extend_from_slice(&htyp2.to_be_bytes());
            msg.push(0); // MCNT
            msg.extend_from_slice(&(total_len as u16).to_be_bytes());

            // MSIN (LOG message, INFO level: MSTP=0, MTIN=4)
            // v2 MSIN: bit 0 reserved, bits 1-3 MSTP, bits 4-7 MTIN
            let msin: u8 = (0x00 << 1) | (0x04 << 4); // LOG + INFO
            msg.push(msin);
            msg.push(self.noar); // NOAR

            // TMSP2 (9 bytes of zeros — timestamp not needed for tracer bullet)
            msg.extend_from_slice(&[0u8; 9]);

            // --- Extension header ---
            if has_wacid {
                // APID
                if let Some(apid) = self.apid {
                    msg.push(4); // length
                    msg.extend_from_slice(&apid);
                } else {
                    msg.push(0); // absent
                }

                // CTID
                if let Some(ctid) = self.ctid {
                    msg.push(4); // length
                    msg.extend_from_slice(&ctid);
                } else {
                    msg.push(0); // absent
                }
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
    use std::io::Write;

    #[test]
    fn v2_roundtrip_single_message() {
        let msg_bytes = V2MessageBuilder::new()
            .with_apid("APP1")
            .with_ctid("CTX1")
            .with_storage_timestamp(1000, 500_000)
            .with_verbose_string("hello")
            .build();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dlt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&msg_bytes).unwrap();
        }

        let dlt = Dlt::open(vec![path]).unwrap();

        assert_eq!(dlt.len(), 1);
        assert!(!dlt.is_empty());
        assert_eq!(dlt.apid(0), "APP1");
        assert_eq!(dlt.ctid(0), "CTX1");
        assert_eq!(
            dlt.storage_timestamp_ns(0),
            1000 * 1_000_000_000 + 500_000
        );

        // Payload should contain the verbose string argument bytes
        let raw = dlt.payload_raw(0);
        assert!(!raw.is_empty());
    }

    #[test]
    fn v2_open_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.dlt");
        std::fs::File::create(&path).unwrap();

        let dlt = Dlt::open(vec![path]).unwrap();
        assert_eq!(dlt.len(), 0);
        assert!(dlt.is_empty());
    }

    #[test]
    fn v2_open_v1_file_returns_zero_messages() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/testfile_control_messages.dlt");

        let dlt = Dlt::open(vec![path]).unwrap();
        // v1 messages should be silently skipped
        assert_eq!(dlt.len(), 0);
    }
}
