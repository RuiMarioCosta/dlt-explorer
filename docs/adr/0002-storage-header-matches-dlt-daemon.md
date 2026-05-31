# Storage header format matches dlt-daemon's DltStorageHeader

The DLT file storage header is 16 bytes and is identical across protocol versions. Its format is defined by dlt-daemon (`dlt_common.h`), not by the AUTOSAR PRS protocol spec. The layout is: pattern (4B, `DLT\x01`) + seconds since epoch (u32 LE) + microseconds (i32 LE) + ECU ID (4B ascii). The AUTOSAR PRS only defines the on-wire message format; the storage header is a file-level concern added by the logging daemon.

## Consequences

- The 9-byte TMSP2 encode/decode functions are exclusively for the in-message base header timestamp (per AUTOSAR PRS), never for the storage header.
- Storage header timestamp resolution is microseconds, not nanoseconds. We convert to nanoseconds internally (`µs * 1000`) for uniform handling alongside the in-message TMSP2 nanosecond timestamps.
