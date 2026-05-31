# Organize DLT modules into v1/ and v2/ subdirectories with shared top-level modules

DLT v1 and v2 have fundamentally different header layouts, field semantics, and parsing strategies — but they share storage header format, payload argument type encoding, MSIN bitfield layout, and service-ID/return-type tables. We split the `src/dlt/` tree into `v1/` and `v2/` subdirectories for version-specific code, keeping genuinely shared definitions in top-level modules (`protocol.rs` for header/framing, `payload.rs` for payload type constants and lookup tables, `intern.rs` for string interning). The legacy `dlt_common.rs` and `dlt_protocol.rs` files (C-style dlt-daemon ports) are retired; their contents are redistributed into the shared modules with shorter, Rust-idiomatic names (`msin_mstp` instead of `dlt_get_msin_mstp`).

## Considered Options

- **Flat files with `v1_`/`v2_` naming prefixes** (e.g. `v2_framer.rs`, `v2_header.rs`). Rejected because v1 will eventually grow beyond a single file too, and the prefix convention doesn't scale — it just delays the directory structure while making every filename longer.
- **Single merged `protocol.rs` for all constants and helpers**. Rejected because payload argument encoding (~100 lines of `TYPE_INFO_*`, `TYLE_*`, `SCOD_*` constants plus lookup tables) is a different layer from header/framing. Mixing them makes `protocol.rs` ~500+ lines with two unrelated concerns.
- **Shared `error.rs`** with a combined `ParseErrorKind` enum covering both versions. Rejected because error variants are tightly coupled to the parser that produces them; a shared enum would require version-prefixed variants (`V1HeaderTooShort`, `V2InvalidExtensionField`) for no practical benefit.

## Consequences

- Each version's parser can evolve independently — adding v2 extension fields or fixing v1 payload panics doesn't touch the other version's code.
- v1 is moved as-is into `v1/mod.rs` without internal restructuring; extracting its payload logic is a separate future change.
- The `dlt_`-prefixed naming convention from the C dlt-daemon port is retired everywhere.
