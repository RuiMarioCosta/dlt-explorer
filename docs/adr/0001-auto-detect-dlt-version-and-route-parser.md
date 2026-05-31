# Auto-detect DLT version and route to the correct parser

DLT files can be either v1 (AUTOSAR 4.x) or v2 (AUTOSAR PRS) format, but the storage header is identical for both — only the base header differs. We detect the version by reading the HTYP byte at offset 16 (immediately after the 16-byte storage header) and extracting bits 5-7. Based on the version, we route to the v1 parser (`v1::Dlt::from_files`) or the v2 parser (`Dlt::open`). Mixed-version file sets are rejected with an explicit error.

## Considered Options

- **Unified parser** that handles both v1 and v2 inline. Rejected because the two protocols have fundamentally different header layouts, field semantics, and payload formats — a combined parser would be complex and fragile.
- **User-specified flag** (e.g. `--v1` / `--v2`). Rejected because the version is trivially detectable from the data, and requiring the user to know which version their files use adds friction for no benefit.
