# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-09-01

### Added

- Rust CLI to generate HTML invoices from YAML using Tera templates
- Input modes: single file, glob pattern, or directory (recursive)
- Bundled default HTML invoice template and custom template support
- Invoice YAML schema validation with totals and line-item checks
- Configurable output filenames via Tera templates
- Default output naming: `[company_slug]-[yyyymmdd]-[invoice_number].html`
- Regression and unit test suite with YAML fixtures
- GitHub Actions CI and release workflows
- Prebuilt binaries for Linux (`x86_64`, `aarch64`) and Windows (`x86_64`)

[Unreleased]: https://github.com/rconnelly/invoicemd-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rconnelly/invoicemd-cli/releases/tag/v0.1.0
