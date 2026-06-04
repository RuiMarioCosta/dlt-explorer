# dlt-explorer

[![ci](https://github.com/RuiMarioCosta/dlt-explorer/actions/workflows/ci.yml/badge.svg)](https://github.com/RuiMarioCosta/dlt-explorer/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/RuiMarioCosta/dlt-explorer/branch/main/graph/badge.svg)](https://codecov.io/gh/RuiMarioCosta/dlt-explorer)
[![CodeQL](https://github.com/RuiMarioCosta/dlt-explorer/actions/workflows/codeql.yml/badge.svg)](https://github.com/RuiMarioCosta/dlt-explorer/actions/workflows/codeql.yml)

## About dlt-explorer

App for visualizing and exploring DLT files.

## Getting Started

### Prerequisites

- Rust toolchain (edition 2024)
- Cargo (installed with Rust)

Install Rust via <https://rustup.rs/>.

## Building

Build in debug mode:

```bash
cargo build
```

Build in release mode:

```bash
cargo build --release
```

## Running

Run with default behavior (GUI mode):

```bash
cargo run
```

Run in terminal mode (see available CLI options):

```bash
cargo run -- --help
```

## Testing

Run all tests:

```bash
cargo test
```

Run only the parity contract validator (checks required v1/v2 benchmark and test pairs):

```bash
cargo test parity_manifest_contract_is_satisfied
```

Edit [parity_manifest.toml](parity_manifest.toml) to add or update required shared scenarios.

Run benchmarks:

```bash
cargo bench
```

## License

This project is licensed under the MIT License. See `LICENSE` for details.



