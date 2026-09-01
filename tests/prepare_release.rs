//! Tests for scripts/prepare-release.py used by semantic-release.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/prepare-release.py")
}

fn write_min_tree(root: &std::path::Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"invoicemd-cli\"\nversion = \"0.2.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("Cargo.lock"),
        "[[package]]\nname = \"invoicemd-cli\"\nversion = \"0.2.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("README.md"),
        "https://github.com/rconnelly/invoicemd-cli/releases/download/v0.2.0/invoicemd-cli-0.2.0-x86_64-unknown-linux-gnu.tar.gz\n./invoicemd-cli-0.2.0-x86_64-unknown-linux-gnu/invoicemd-cli --version\n",
    )
    .unwrap();
    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## [Unreleased]\n\n[Unreleased]: https://github.com/rconnelly/invoicemd-cli/compare/v0.2.0...HEAD\n[0.2.0]: https://github.com/rconnelly/invoicemd-cli/compare/v0.1.0...v0.2.0\n",
    )
    .unwrap();
}

#[test]
fn prepare_release_bumps_version_files() {
    let dir = tempdir().unwrap();
    write_min_tree(dir.path());

    let status = Command::new("python3")
        .arg(script())
        .arg("0.3.0")
        .arg("0.2.0")
        .env("INVOICEMD_ROOT", dir.path())
        .status()
        .expect("python3 should run prepare-release.py");
    assert!(status.success(), "prepare-release.py failed");

    let cargo = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("version = \"0.3.0\""));

    let lock = fs::read_to_string(dir.path().join("Cargo.lock")).unwrap();
    assert!(lock.contains("version = \"0.3.0\""));

    let readme = fs::read_to_string(dir.path().join("README.md")).unwrap();
    assert!(readme.contains("/download/v0.3.0/invoicemd-cli-0.3.0-"));
    assert!(readme.contains("invoicemd-cli-0.3.0-x86_64-unknown-linux-gnu"));

    let changelog = fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
    assert!(changelog.contains("compare/v0.3.0...HEAD"));
    assert!(changelog
        .contains("[0.3.0]: https://github.com/rconnelly/invoicemd-cli/compare/v0.2.0...v0.3.0"));
}
