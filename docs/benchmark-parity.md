# Benchmark parity naming contract

This repository uses an operation-first benchmark layout under a single Criterion entrypoint in `benches/dlt.rs`.

## Frozen naming convention

Benchmark IDs are frozen to the following canonical format:

- Group: operation name (`scan_frames`, `parse_header`, `decode_payload`, `open_file`)
- Benchmark identifier: `<protocol>_<operation>` (`v1_scan_frames`, `v2_scan_frames`, etc.)
- Benchmark parameter: shared scenario name (`uniform_ecu_small`, `uniform_ecu_large`, `sparse_mixed_ecu_large`, `dense_mixed_ecu_large`, `marker_in_payload`, `truncated_tail`)

Example:

- `BenchmarkId::new("v1_scan_frames", "uniform_ecu_large")`
- `BenchmarkId::new("v2_decode_payload", "marker_in_payload")`

## Migration note

Issue #74 performed the one-time rename from legacy benchmark IDs to canonical IDs.
Future additions must follow this contract and be declared in `parity_manifest.toml`.
