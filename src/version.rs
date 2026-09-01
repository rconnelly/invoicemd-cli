//! Crate and build metadata exposed to the CLI.

/// Semantic version from `Cargo.toml` (e.g. `0.1.0`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short git commit used at build time, or `unknown` when unavailable.
pub const GIT_SHA: &str = env!("BUILD_GIT_SHA");

/// Value shown for `invoicemd-cli --version`.
pub const LONG_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("BUILD_GIT_SHA"), ")");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        let parts: Vec<_> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "VERSION should be major.minor.patch");
        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "version parts should be numeric"
            );
        }
    }

    #[test]
    fn long_version_includes_crate_version() {
        assert!(LONG_VERSION.starts_with(VERSION));
    }
}
