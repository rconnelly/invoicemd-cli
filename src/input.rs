use anyhow::{bail, Context, Result};
use glob::glob;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const YAML_EXTENSIONS: &[&str] = &["yaml", "yml"];

pub fn collect_yaml_paths(inputs: &[String]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for input in inputs {
        let expanded = expand_input(input)?;
        paths.extend(expanded);
    }

    paths.sort();
    paths.dedup();

    if paths.is_empty() {
        bail!("no YAML invoice files found for the given input(s)");
    }

    Ok(paths)
}

fn expand_input(input: &str) -> Result<Vec<PathBuf>> {
    let path = Path::new(input);

    if path.is_dir() {
        return yaml_files_in_dir(path);
    }

    if path.is_file() {
        ensure_yaml(path)?;
        return Ok(vec![path.to_path_buf()]);
    }

    if input.contains('*') || input.contains('?') || input.contains('[') {
        return yaml_files_from_glob(input);
    }

    bail!(
        "input '{}' is not a file, directory, or glob pattern",
        input
    );
}

fn yaml_files_in_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() && is_yaml(entry_path) {
            files.push(entry_path.to_path_buf());
        }
    }

    Ok(files)
}

fn yaml_files_from_glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in glob(pattern).with_context(|| format!("invalid glob pattern '{pattern}'"))? {
        let path = entry.with_context(|| format!("failed to read glob match for '{pattern}'"))?;
        if path.is_file() {
            ensure_yaml(&path)?;
            files.push(path);
        }
    }

    Ok(files)
}

fn ensure_yaml(path: &Path) -> Result<()> {
    if is_yaml(path) {
        Ok(())
    } else {
        bail!("{} is not a YAML file (.yaml or .yml)", path.display())
    }
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| YAML_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_yaml_from_directory() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.yaml"), "invoice: {}\n").unwrap();
        fs::write(dir.path().join("ignore.txt"), "nope").unwrap();

        let paths = yaml_files_in_dir(dir.path()).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("a.yaml"));
    }
}
