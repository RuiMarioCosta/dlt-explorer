/// A structured parse error for a single malformed DLT frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Index of the source file in the paths list.
    pub file_index: u16,
    /// Byte offset within the file where the error was detected.
    pub byte_offset: u64,
    /// What went wrong.
    pub kind: ParseErrorKind,
}

/// Classification of parse errors encountered during DLT frame scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Data ended before a complete header or message could be read.
    Truncated,
    /// Base header version field is not the expected version.
    InvalidVersion { found: u8 },
    /// Standard header fields/flags are inconsistent or malformed.
    InvalidStandardHeader,
    /// Declared message length does not match available data.
    LengthMismatch { declared: u16, available: usize },
    /// An extension header field could not be parsed.
    InvalidExtensionField,
    /// Payload offset or length exceeds the message bounds.
    PayloadOutOfBounds,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "file {}, offset {:#x}: {:?}",
            self.file_index, self.byte_offset, self.kind
        )
    }
}
