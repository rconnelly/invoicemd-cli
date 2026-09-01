# Releasing

This project uses [Semantic Versioning](https://semver.org/) and GitHub Releases with prebuilt binaries.

## Version sources

| Location | Purpose |
| --- | --- |
| `Cargo.toml` `version` | Canonical crate / CLI version |
| `CHANGELOG.md` | Human-readable release notes |
| Git tag `vX.Y.Z` | Triggers the release workflow |

The CLI reports `CARGO_PKG_VERSION` and a short git commit via `--version`.

## Release checklist

1. **Update `CHANGELOG.md`**
   - Move items from `[Unreleased]` into a new `## [X.Y.Z] - YYYY-MM-DD` section.

2. **Bump the version in `Cargo.toml`**
   - Set `version = "X.Y.Z"` to match the tag you will create (without the `v` prefix).

3. **Commit on `main`**
   ```bash
   git checkout main
   git pull
   git add CHANGELOG.md Cargo.toml
   git commit -m "Release vX.Y.Z"
   ```

4. **Tag and push**
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin main
   git push origin vX.Y.Z
   ```

   Optional local check before pushing the tag:

   ```bash
   ./scripts/verify-release.sh vX.Y.Z
   ```

5. **Verify the GitHub Release**
   - The [Release workflow](.github/workflows/release.yml) validates that the tag matches `Cargo.toml`, extracts the changelog section, builds binaries, and publishes a GitHub Release with assets.

## Tag rules

- Tags must match `vMAJOR.MINOR.PATCH` (for example `v0.1.0`).
- The tag without `v` must exactly match `Cargo.toml` `version`.
- A matching `## [X.Y.Z]` section must exist in `CHANGELOG.md`.

## Release artifacts

Each release publishes archives for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-pc-windows-msvc`

Archive names follow:

```text
invoicemd-cli-<version>-<target>.tar.gz
invoicemd-cli-<version>-<target>.zip   # Windows
```

Each archive includes the binary, `README.md`, `CHANGELOG.md`, `templates/default.html`, and `examples/*.yaml`.

## Local verification before tagging

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release
./target/release/invoicemd-cli --version
```

## Installing from a release

Download the archive for your platform from GitHub Releases, extract it, and run `invoicemd-cli` from the extracted directory (or move the binary onto your `PATH`).
