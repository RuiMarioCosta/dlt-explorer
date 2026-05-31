pub mod v1;
pub mod v2;
pub mod error;
pub mod intern;
pub mod payload;
pub mod protocol;

use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Detect the DLT protocol version from the first message in a file.
///
/// Reads the HTYP byte immediately after the v1 storage header (16 bytes)
/// and extracts the version from bits 5-7.  Returns 1 for v1 files and
/// 2 for v2 files.
pub fn detect_version(path: &PathBuf) -> Result<u8> {
    let mut file = File::open(path)?;
    // v1 storage header: DLT\x01 (4) + seconds (4) + microseconds (4) + ECU (4) = 16
    // The HTYP byte follows at offset 16.
    let mut buf = [0u8; 17];
    file.read_exact(&mut buf)?;
    if &buf[0..4] != b"DLT\x01" {
        anyhow::bail!("Not a DLT file: missing DLT\\x01 marker");
    }
    let htyp = buf[16];
    let version = (htyp >> 5) & 0x07;
    Ok(version)
}
