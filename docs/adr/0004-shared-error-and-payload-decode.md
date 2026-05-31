# Shared error module for both DLT protocol versions

## Status

Accepted

## Context

ADR-0003 rejected a shared `error.rs` module because error variants were expected to be tightly coupled to each version's parser. In practice, both v1 and v2 parsers emit the same core error variants: `Truncated`, `InvalidVersion`, `LengthMismatch`, `InvalidExtensionField`, and `PayloadOutOfBounds`. Duplicating these across version-specific modules adds maintenance burden without benefit.

Additionally, payload decode logic (verbose, non-verbose, control) is identical between versions — only the byte order differs. Moving decode functions to the shared `dlt/payload.rs` with an explicit `big_endian: bool` parameter enables code reuse without version coupling.

## Decision

- Lift `ParseError` and `ParseErrorKind` from `v2/error.rs` to a shared `dlt/error.rs` module.
- `v2/error.rs` becomes a re-export of the shared types.
- Move `decode_verbose`, `decode_non_verbose`, `decode_control`, and `hex_dump` from `v2/payload.rs` to the shared `dlt/payload.rs`, adding a `big_endian: bool` parameter for all multi-byte reads.
- `v2/payload.rs` becomes a thin dispatch wrapper calling shared functions with `big_endian=true`.
- Add `DLT_HTYP_MSBF` constant and `htyp_has_msbf()` helper to `protocol.rs` so v1 callers can determine byte order from HTYP.

## Consequences

- Both versions share one error type hierarchy — new error variants benefit both parsers.
- Payload decode logic is maintained in a single location; byte-order differences are handled by a parameter rather than code duplication.
- v1 payload decoding (future work) can call the same shared functions with the MSBF flag from its HTYP byte.
- This reverses the ADR-0003 rejection of shared errors, since real-world usage proved the variants are genuinely common.
