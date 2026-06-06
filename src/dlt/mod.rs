pub mod error;
pub mod intern;
pub mod payload;
pub mod storage;
pub mod v1;
pub mod v2;

use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::dlt::storage::{STORAGE_HEADER_PATTERN, STORAGE_HEADER_SIZE};

/// Detect the DLT protocol version from the first message in a file.
///
/// Reads the Header Type byte immediately after the storage header (16 bytes)
/// and extracts the version from bits 5-7.  Returns 1 for v1 files and
/// 2 for v2 files.
pub fn detect_version(path: &PathBuf) -> Result<u8> {
    let mut file = File::open(path)?;
    // Read storage header plus the first message header byte (HTYP).
    let mut buf = [0u8; STORAGE_HEADER_SIZE + 1];
    file.read_exact(&mut buf)?;
    if &buf[0..4] != STORAGE_HEADER_PATTERN {
        anyhow::bail!("Not a DLT file: missing DLT\\x01 marker");
    }
    let htyp = buf[STORAGE_HEADER_SIZE];
    let version = (htyp >> 5) & 0x07;
    Ok(version)
}
