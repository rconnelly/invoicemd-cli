#!/usr/bin/env python3
"""Bump crate version files for a semantic-release or manual prepare step."""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(os.environ.get("INVOICEMD_ROOT", Path(__file__).resolve().parent.parent))
REPO = "https://github.com/rconnelly/invoicemd-cli"


def replace_once(path: Path, pattern: str, replacement: str, flags: int = 0) -> None:
    text = path.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise SystemExit(f"{path}: expected 1 replacement for {pattern!r}, got {count}")
    path.write_text(updated)


def bump_cargo(version: str) -> None:
    replace_once(
        ROOT / "Cargo.toml",
        r'(?m)^version = "[^"]+"',
        f'version = "{version}"',
    )
    replace_once(
        ROOT / "Cargo.lock",
        r'(name = "invoicemd-cli"\n)version = "[^"]+"',
        rf'\1version = "{version}"',
    )


def bump_readme(version: str) -> None:
    path = ROOT / "README.md"
    text = path.read_text()
    text = re.sub(
        r"releases/download/v\d+\.\d+\.\d+/invoicemd-cli-\d+\.\d+\.\d+-",
        f"releases/download/v{version}/invoicemd-cli-{version}-",
        text,
    )
    text = re.sub(
        r"invoicemd-cli-\d+\.\d+\.\d+-x86_64-unknown-linux-gnu",
        f"invoicemd-cli-{version}-x86_64-unknown-linux-gnu",
        text,
    )
    path.write_text(text)


def bump_changelog_links(version: str, previous: str | None) -> None:
    path = ROOT / "CHANGELOG.md"
    text = path.read_text()
    text = re.sub(
        r"\[Unreleased\]: .+",
        f"[Unreleased]: {REPO}/compare/v{version}...HEAD",
        text,
        count=1,
    )
    new_link = (
        f"[{version}]: {REPO}/compare/v{previous}...v{version}"
        if previous
        else f"[{version}]: {REPO}/releases/tag/v{version}"
    )
    marker = f"[{version}]:"
    if marker not in text:
        text = re.sub(
            r"\[Unreleased\]: .+\n",
            rf"[Unreleased]: {REPO}/compare/v{version}...HEAD\n{new_link}\n",
            text,
            count=1,
        )
    path.write_text(text)


def main(argv: list[str]) -> None:
    if len(argv) < 2 or not argv[1].strip():
        raise SystemExit("Usage: prepare-release.py <version> [previous-version]")
    version = argv[1].strip()
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise SystemExit(f"version must look like 1.2.3, got {version!r}")
    previous = argv[2].strip() if len(argv) > 2 and argv[2].strip() else None
    if previous in {"0.0.0", "undefined", "null"}:
        previous = None
    bump_cargo(version)
    bump_readme(version)
    bump_changelog_links(version, previous)


if __name__ == "__main__":
    main(sys.argv)
