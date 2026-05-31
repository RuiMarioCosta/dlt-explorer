# DLT Explorer

A parser and viewer for AUTOSAR Diagnostic Log and Trace (DLT) files supporting both v1 and v2 protocol formats.

## Language

**Storage Header**:
A 16-byte file-level header prepended to each message by the dlt-daemon when writing to disk. Contains receive timestamp (seconds + microseconds) and ECU ID. Defined by dlt-daemon, not AUTOSAR PRS.
_Avoid_: file header, frame header

**Base Header**:
The protocol-defined header at the start of each DLT message (after the storage header in files). Contains version, message counter, length, and conditionally MSIN, NOAR, TMSP2, MSID.
_Avoid_: standard header (v1 term), message header

**TMSP2**:
A 9-byte nanosecond-resolution timestamp in the v2 base header (4B nanoseconds BE + 5B seconds BE). Only present in data messages (verbose or non-verbose).
_Avoid_: storage timestamp, file timestamp

**HTYP / HTYP2**:
The header type field. HTYP is the 1-byte v1 version; HTYP2 is the 4-byte v2 version. Contains protocol version, flags indicating which optional fields are present.

**CNTI**:
Content Info — a 2-bit field in HTYP2 indicating whether the message is verbose (0), non-verbose (1), or control (2).
_Avoid_: message mode, verbosity flag

**ECU**:
Electronic Control Unit — the source device identifier. A 4-byte null-padded ASCII string.
_Avoid_: device, node

**APID**:
Application ID — identifies the software component that produced the message. 4-byte null-padded ASCII.
_Avoid_: app name, source

**CTID**:
Context ID — identifies a logical logging context within an application. 4-byte null-padded ASCII.
_Avoid_: channel, category
