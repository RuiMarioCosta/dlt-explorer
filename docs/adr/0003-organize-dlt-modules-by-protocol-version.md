# Organize DLT protocol ownership by version with storage-only sharing

## Status

Accepted

## Context

The repository already separates parser entry points into `src/dlt/v1/` and `src/dlt/v2/`, but message-level protocol semantics had remained partially centralized in a mixed top-level `src/dlt/protocol.rs` module.

That mixed ownership made it hard to reason about where helpers belong:

- Storage Header concerns (file framing and on-disk metadata) are genuinely shared.
- Base Header semantics (`HTYP` vs `HTYP2`, `MSIN`, `CNTI`, `TMSP2`) are version-owned and should live with the parser that consumes them.

Keeping these together in one shared protocol module blurred boundaries and made review/navigation harder.

## Decision

- Keep sharing only file-level Storage Header concepts at the top level in `src/dlt/storage.rs`:
	- delimiter pattern (`DLT\x01`)
	- storage field sizes and aggregate storage header size
- Move all message-level protocol semantics to owning versions:
	- `src/dlt/v1/protocol.rs` owns v1 `HTYP` flags, v1 `MSIN` helpers, and v1 message classification helpers.
	- `src/dlt/v2/protocol.rs` owns v2 `HTYP2`/`CNTI`/`TMSP2` constants and helpers, plus v2 message-construction helpers used by tests.
- Keep top-level version detection as a narrow bootstrap exception in `src/dlt/mod.rs`:
	- It may inspect only the minimal common prefix needed to dispatch.
	- It must not grow into shared message-level protocol logic.
- Perform this as a single cutover and remove the mixed shared protocol module.
- Allow deliberate duplication of tiny helpers across v1/v2 when it preserves ownership clarity.

## Consequences

- Maintainers can work on version-specific protocol behavior in one place per version.
- Storage Header framing remains explicitly shared and isolated from message semantics.
- Parser-root responsibilities remain orchestration and dispatch only.
- The code layout now matches the documented ownership rule, reducing ambiguity for future refactors.
