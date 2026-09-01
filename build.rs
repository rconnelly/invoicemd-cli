fn main() {
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("GITHUB_SHA")
                .ok()
                .map(|value| value.chars().take(7).collect())
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=BUILD_GIT_SHA={sha}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}
