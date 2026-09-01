mod cli;
mod input;
mod invoice;
mod render;

use anyhow::{Context, Result};
use cli::Cli;
use clap::Parser;
use input::collect_yaml_paths;
use invoice::InvoiceDocument;
use render::{
    build_renderer, render_invoice_html, render_output_filename, DEFAULT_FILENAME_TEMPLATE,
};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    let yaml_paths = collect_yaml_paths(&cli.inputs)?;
    let renderer = build_renderer(cli.template.as_deref())?;

    for yaml_path in yaml_paths {
        process_one(
            &yaml_path,
            &renderer,
            cli.output_dir.as_deref(),
            cli.output_name.as_deref(),
        )
        .with_context(|| format!("failed to generate invoice from {}", yaml_path.display()))?;
    }

    Ok(())
}

fn process_one(
    yaml_path: &Path,
    renderer: &render::Renderer,
    output_dir: Option<&Path>,
    global_filename_template: Option<&str>,
) -> Result<()> {
    let doc = InvoiceDocument::from_yaml_file(yaml_path)?;
    let html = render_invoice_html(renderer, &doc)?;

    let filename_template = global_filename_template
        .or(doc.output.filename.as_deref())
        .unwrap_or(DEFAULT_FILENAME_TEMPLATE);

    let filename = render_output_filename(&doc, filename_template)?;
    let output_path = resolve_output_path(yaml_path, output_dir, &filename);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    std::fs::write(&output_path, html)
        .with_context(|| format!("failed to write HTML file {}", output_path.display()))?;

    println!("Wrote {}", output_path.display());
    Ok(())
}

fn resolve_output_path(yaml_path: &Path, output_dir: Option<&Path>, filename: &str) -> PathBuf {
    match output_dir {
        Some(dir) => dir.join(filename),
        None => yaml_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(filename),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn end_to_end_single_file() {
        let dir = tempdir().unwrap();
        let yaml_path = dir.path().join("invoice.yaml");
        fs::write(
            &yaml_path,
            r#"invoice:
  number: 42
  date: 2026-01-15
  currency: USD
company:
  name: Beta Labs LLC
  address: |
    100 Market Street
    San Francisco, CA
client:
  name: Example Client
  address: 200 Pine Street
line_items:
  - description: Design review
    quantity: 3
    unit_price: "150.00"
totals:
  tax_rate: "0.0825"
notes:
  - Thank you for your business.
"#,
        )
        .unwrap();

        let out_dir = dir.path().join("out");
        let cli = Cli {
            template: None,
            output_dir: Some(out_dir.clone()),
            output_name: None,
            inputs: vec![yaml_path.to_string_lossy().to_string()],
        };

        run(cli).unwrap();

        let expected = out_dir.join("betal-20260115-0042.html");
        assert!(expected.exists(), "expected output at {}", expected.display());
        let html = fs::read_to_string(expected).unwrap();
        assert!(html.contains("Beta Labs LLC"));
        assert!(html.contains("Invoice #42"));
    }
}
