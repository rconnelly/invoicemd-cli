# invoicemd-cli

A Rust CLI that creates invoices from markdown file data. Supports HTML and PDF layouts. Also manages invoice storage for git support.

## Cursor Cloud specific instructions

### Project state

Rust CLI (`invoicemd-cli`) that renders HTML and PDF invoices from YAML via Tera. Entry point: `src/main.rs`. Bundled template: `templates/default.html`. Sample data: `examples/`.

### Toolchain

The Rust toolchain is preinstalled in the base image. Use `rustup default stable` if `rustc` is older than 1.88 (required by `printpdf` 0.12 and recent dependencies). Verified with `rustc`/`cargo` 1.98.0.

Tools on `PATH`: `rustc`, `cargo`, `rustfmt`, `clippy` at `/usr/local/cargo/bin`.

### Standard commands

- Build (dev): `cargo build`
- Run the CLI: `cargo run -- [OPTIONS] <INPUT>...` (see `cargo run -- --help`)
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format check: `cargo fmt --check`
- Release process: see `RELEASING.md` at repo root (tag `vX.Y.Z` triggers `.github/workflows/release.yml`)

Example end-to-end:

```bash
cargo run -- -d /tmp/out examples/
```

This is a local one-shot CLI — no long-running services, servers, databases, or ports.
