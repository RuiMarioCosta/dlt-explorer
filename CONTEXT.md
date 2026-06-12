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

**MSIN**:
Message Info — a 1-byte field in the extended header that encodes VERB, MSTP, and MTIN.
_Avoid_: log level byte

**MSTP**:
Message Type — a 3-bit value encoded in MSIN that selects the family (log, app_trace, nw_trace, control).
_Avoid_: message category, log type

**MTIN (Message Type Info)**:
A 4-bit value encoded in MSIN whose meaning depends on MSTP (for example log level for log messages, service direction for control messages).
_Avoid_: log level (as a generic term)

**ECU**:
Electronic Control Unit — the source device identifier. A 4-byte null-padded ASCII string.
_Avoid_: device, node

**APID**:
Application ID — identifies the software component that produced the message. 4-byte null-padded ASCII.
_Avoid_: app name, source

**CTID**:
Context ID — identifies a logical logging context within an application. 4-byte null-padded ASCII.
_Avoid_: channel, category

**Log Table**:
The primary message list used to explore DLT traffic. Each row represents one DLT message and exposes key fields such as timestamp, ECU, APID, CTID, type, and payload.
_Avoid_: raw row buffer, UI cache

**Viewport Rendering**:
Rendering only the subset of Log Table rows that are currently visible to the user, instead of rendering the entire loaded message set.
_Avoid_: full-table rendering, eager row painting

**Retained Data Layer**:
The application-owned in-memory representation of loaded DLT data and derived message metadata, independent of any specific GUI framework.
_Avoid_: widget state, UI row model

**Index Layer**:
Derived lookup structures that support fast filtering, search, sorting, and index-based access over loaded DLT data.
_Avoid_: cache (as a generic term), UI state

**Application Layer**:
The state-transition layer that coordinates user intents, loading lifecycle, query application, and selection state over retained data.
_Avoid_: widget state, parser layer

**Desktop UI Layer**:
The egui-based viewport layer that renders the Log Table and emits interaction intents without owning parsing, query, or retained data logic.
_Avoid_: data owner, business logic layer

**Load Job**:
An asynchronous request to load selected DLT files and produce a Retained Data Layer snapshot for the application state to adopt.
_Avoid_: blocking file open, UI thread parse

**Load Generation**:
A monotonic token attached to each Load Job so the Application Layer can ignore stale completion events and apply only the latest accepted result.
_Avoid_: implicit race ordering, first-finished-wins

**Intent/Event Boundary**:
The explicit contract where the Desktop UI Layer emits user intents and the Application Layer applies state transitions and emits resulting state.
_Avoid_: direct UI mutation, callback side effects

**Selection Continuity**:
The rule that preserves selected message identity across query updates when still visible, otherwise selects the first visible row or clears selection when no rows remain.
_Avoid_: unconditional clear, position-only selection

**Query Pipeline**:
The ordered query flow where Structured Filter narrows rows first and Rendered Text Search refines the visible subset second.
_Avoid_: merged opaque query, undefined query order

**Structured Filter**:
A filter expressed against parsed DLT fields such as ECU, APID, CTID, message type, timestamps, or decoded argument values.
_Avoid_: text search, grep filter

**Rendered Text Search**:
A search over the user-visible textual representation of a DLT message, including rendered payload text shown in the UI.
_Avoid_: structured filter, raw byte search
