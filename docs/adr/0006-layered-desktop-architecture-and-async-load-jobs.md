# Layered desktop architecture and async load jobs

## Status

Accepted

## Decision

- Keep the desktop path standardized on egui under a strict layer split: Desktop UI Layer, Application Layer, and Retained Data plus Index layers.
- Keep loading asynchronous via worker-thread Load Jobs.
- Keep Load Generation latest-wins gating as the authoritative stale-result policy.
- Keep the current supersede-only behavior: a newer Load Job supersedes older jobs for result adoption, but older jobs are not hard-cancelled.
- Keep the explicit Query Pipeline ordering (Structured Filter first, Rendered Text Search second) and Selection Continuity behavior in the Application and Retained layers.

## Current baseline: supersede without hard cancellation

- Starting a new Load Job increments Load Generation and makes previous in-flight generations stale for adoption.
- Stale `LoadCompleted` and `LoadFailed` events are ignored by the Application Layer.
- Worker threads for stale generations are allowed to finish naturally; there is no cooperative cancellation signal yet.
- This intentionally keeps the Intent/Event Boundary simple and deterministic while avoiding cross-layer cancellation plumbing in the current architecture.

## Revisit trigger: when to add cooperative cancellation

Introduce cooperative cancellation only when profiling shows supersede-only behavior is a material cost under representative desktop workloads.

Trigger a cancellation design revisit when both conditions hold in the same profiling cycle:

1. Stale-generation parsing and decode work accounts for >= 20% of total load-job CPU time.
2. Cancelling stale generations is projected (prototype or instrumented estimate) to improve p95 latest-load completion latency by >= 15%.

Profiling cycle requirements:

- Use benchmark-parity fixture scenarios for reproducible input shape coverage.
- Include at least one rapid user-intent supersede pattern (for example, back-to-back file opens where the first generation becomes stale).
- Run at least 10 iterations per scenario and report median and p95 latency plus stale-work share.

If the trigger is met, create a follow-up ADR that preserves:

- The Intent/Event Boundary ownership model.
- Load Generation latest-wins correctness.
- The layered separation between Desktop UI, Application, and Retained/Index logic.

## Consequences

- Current behavior stays predictable and easier to test: stale completions cannot overwrite newer retained state.
- The architecture remains explicit that supersede-only is an intentional baseline, not an accidental omission.
- Future cancellation work is gated by objective measurements instead of premature complexity.
