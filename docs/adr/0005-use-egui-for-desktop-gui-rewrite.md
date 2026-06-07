# Use egui for the desktop GUI rewrite

## Status

Accepted

## Context

The current GUI is an early iced-based prototype that can open a DLT file and render a simple message list. The project goal is to evolve into a desktop log-analysis tool that is very similar to dlt-viewer while improving performance for large files, filtering, and search.

The key requirements for the next GUI stack are:

- Cross-platform desktop support on Windows, Linux, and macOS.
- A Rust-native stack unless a non-Rust alternative is overwhelmingly better.
- Fast iteration on dense tooling-oriented workflows rather than native widget fidelity.
- A main Log Table that can scale to very large message sets through Viewport Rendering.
- A thin GUI layer over a Retained Data Layer and Index Layer that own parsing, filtering, search, and navigation.

Three realistic options were considered:

- Continue with iced.
- Rewrite the desktop UI with egui and eframe.
- Leave the pure Rust GUI ecosystem for a Qt-based solution.

iced remains a viable Rust GUI framework, but it is not the strongest fit for a dlt-viewer-like application centered on a giant interactive table, custom inspection workflows, and rapid iteration on tooling UI patterns. Qt offers mature desktop tables and pane composition, but introduces a significantly heavier build, deployment, and long-term integration story for a Rust-first project.

## Decision

- Rewrite the desktop GUI on egui with eframe.
- Treat the GUI as a viewport over application-owned data, not as the owner of loaded message state.
- Keep parsing, filtering, search, and navigation in framework-agnostic modules beneath the GUI.
- Prioritize a fast table-centric workflow similar to dlt-viewer before adding more advanced workspace composition.
- Prefer custom log-viewer widgets built from egui primitives over trying to force a generic desktop widget model onto the application.

## Consequences

- The rewrite stays Rust-native and cross-platform while giving the project more flexibility for dense, tooling-oriented UI design than iced.
- The project accepts more ownership of custom table behavior and viewport logic than a traditional model/view desktop framework would provide.
- UI maintainability now depends on keeping egui view code thin and preventing application state, query logic, and rendering concerns from collapsing into one layer.
- If later requirements shift toward native desktop fidelity, full docking workflows, or mature out-of-the-box widget behavior, Qt may need to be reconsidered.
- Existing iced-specific UI code becomes transitional and should not shape the long-term application architecture.