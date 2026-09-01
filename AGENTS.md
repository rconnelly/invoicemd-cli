# invoicemd-cli

A Rust CLI that creates invoices from markdown file data. Supports an HTML layout. Also manages invoice storage for git support.

## Cursor Cloud specific instructions

### Project state

This is a greenfield Rust CLI project. At the time of environment setup the repository contained only `README.md` — there is no `Cargo.toml`, `src/`, or other Rust code yet. Once the project is scaffolded (e.g. via `cargo init`/`cargo new`), the standard Cargo workflow below applies.

### Toolchain

The Rust toolchain is preinstalled in the base image (no install step needed): `rustc`, `cargo`, `rustfmt`, and `clippy` are all available on `PATH` at `/usr/local/cargo/bin`. Verified working with `rustc`/`cargo` 1.83.0.

### Standard commands (once `Cargo.toml` exists)

- Build (dev): `cargo build`
- Run the CLI: `cargo run -- <args>`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format check: `cargo fmt --check` (apply with `cargo fmt`)

This is a local one-shot CLI — there are no long-running services, servers, databases, or ports to start.
