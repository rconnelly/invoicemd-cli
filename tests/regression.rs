//! Regression tests for invoicemd-cli end-to-end behavior and schema validation.

use assert_cmd::Command;
use invoicemd_cli::cli::{Cli, OutputFormat};
use invoicemd_cli::input::collect_yaml_paths;
use invoicemd_cli::invoice::InvoiceDocument;
use invoicemd_cli::render::{
    render_output_filename, DEFAULT_FILENAME_TEMPLATE, DEFAULT_HTML_TEMPLATE,
};
use invoicemd_cli::run;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")
}

#[test]
fn regression_examples_acme_invoice_output() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out");
    fs::create_dir_all(&out).unwrap();

    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![examples_dir()
            .join("acme-invoice.yaml")
            .to_string_lossy()
            .into_owned()],
    })
    .expect("acme example should render");

    let html_path = out.join("acmec-20260315-1042.html");
    assert!(html_path.is_file(), "missing {}", html_path.display());

    let html = fs::read_to_string(&html_path).unwrap();
    for needle in [
        "Acme Corporation",
        "Invoice #1042",
        "Example Client LLC",
        "Product design workshop",
        "USD 5412.50",
        "Thank you for your business.",
    ] {
        assert!(html.contains(needle), "expected HTML to contain: {needle}");
    }
}

#[test]
fn regression_examples_beta_custom_output_filename() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out");

    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![examples_dir()
            .join("beta-invoice.yaml")
            .to_string_lossy()
            .into_owned()],
    })
    .unwrap();

    let html_path = out.join("beta-labs-inv-3.html");
    assert!(html_path.is_file());
    let html = fs::read_to_string(html_path).unwrap();
    assert!(html.contains("Beta Labs"));
    assert!(html.contains("Invoice #3"));
}

#[test]
fn regression_directory_input_processes_all_yaml_files() {
    let dir = tempdir().unwrap();
    let invoices = dir.path().join("invoices");
    fs::create_dir_all(invoices.join("nested")).unwrap();
    fs::copy(fixture("valid/minimal.yaml"), invoices.join("a.yaml")).unwrap();
    fs::copy(
        fixture("valid/full-totals.yaml"),
        invoices.join("nested/b.yml"),
    )
    .unwrap();
    fs::write(invoices.join("readme.txt"), "skip").unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![invoices.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(out.join("betal-20260115-0042.html").is_file());
    assert!(out.join("123nu-20260501-0009.html").is_file());
}

#[test]
fn regression_glob_input_processes_matching_files() {
    let dir = tempdir().unwrap();
    fs::copy(fixture("valid/minimal.yaml"), dir.path().join("one.yaml")).unwrap();
    fs::copy(
        fixture("valid/full-totals.yaml"),
        dir.path().join("two.yml"),
    )
    .unwrap();
    fs::write(dir.path().join("three.txt"), "nope").unwrap();

    let out = dir.path().join("out");
    let pattern = dir.path().join("*.yaml").to_string_lossy().into_owned();

    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![pattern],
    })
    .unwrap();

    assert!(out.join("betal-20260115-0042.html").is_file());
    assert!(!out.join("123nu-20260501-0009.html").exists());
}

#[test]
fn regression_global_output_name_overrides_yaml_setting() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/custom-output.yaml"), &yaml).unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: Some("global-{{ invoice.number }}.html".into()),
        format: vec![OutputFormat::Html],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(out.join("global-12.html").is_file());
    assert!(!out.join("custo-custom-12.html").exists());
}

#[test]
fn regression_yaml_output_filename_template() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/custom-output.yaml"), &yaml).unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(out.join("custo-custom-12.html").is_file());
}

#[test]
fn regression_custom_html_template() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/minimal.yaml"), &yaml).unwrap();

    let template = dir.path().join("custom.html");
    fs::write(
        &template,
        "<html><body><h1>{{ company.name }}</h1></body></html>",
    )
    .unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: Some(template),
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    let html = fs::read_to_string(out.join("betal-20260115-0042.html")).unwrap();
    assert_eq!(html, "<html><body><h1>Beta Labs LLC</h1></body></html>");
}

#[test]
fn regression_invalid_fixtures_fail_validation() {
    for file in [
        "invalid/empty-line-items.yaml",
        "invalid/due-before-issue.yaml",
        "invalid/mismatched-subtotal.yaml",
        "invalid/mismatched-line-amount.yaml",
    ] {
        let path = fixture(file);
        let err = InvoiceDocument::from_yaml_file(&path).expect_err(&format!(
            "expected validation failure for {}",
            path.display()
        ));
        let message = format!("{err:#}");
        assert!(
            !message.is_empty(),
            "expected non-empty error for {}",
            path.display()
        );
    }
}

#[test]
fn regression_collect_yaml_paths_rejects_non_yaml_file() {
    let dir = tempdir().unwrap();
    let txt = dir.path().join("invoice.txt");
    fs::write(&txt, "not yaml").unwrap();

    let err = collect_yaml_paths(&[txt.to_string_lossy().into_owned()]).unwrap_err();
    assert!(format!("{err:#}").contains("not a YAML file"));
}

#[test]
fn regression_collect_yaml_paths_deduplicates_inputs() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/minimal.yaml"), &yaml).unwrap();
    let path = yaml.to_string_lossy().into_owned();

    let paths = collect_yaml_paths(&[path.clone(), path]).unwrap();
    assert_eq!(paths.len(), 1);
}

#[test]
fn regression_empty_directory_input_fails() {
    let dir = tempdir().unwrap();
    let empty = dir.path().join("empty");
    fs::create_dir_all(&empty).unwrap();

    let err = run(Cli {
        template: None,
        output_dir: None,
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![empty.to_string_lossy().into_owned()],
    })
    .unwrap_err();

    assert!(format!("{err:#}").contains("no YAML invoice files found"));
}

#[test]
fn regression_filename_template_rejects_path_separators() {
    let doc = InvoiceDocument::from_yaml_file(&fixture("valid/minimal.yaml")).unwrap();
    let err = render_output_filename(&doc, "{{ company_slug }}/{{ invoice_number }}.html")
        .expect_err("path separators should be rejected");
    assert!(format!("{err:#}").contains("path separators"));
}

#[test]
fn regression_default_template_is_valid_tera() {
    assert!(
        DEFAULT_HTML_TEMPLATE.contains("{% for item in line_items %}"),
        "bundled template should iterate line items"
    );
    assert!(
        DEFAULT_HTML_TEMPLATE.contains("{{ company.name }}"),
        "bundled template should reference company name"
    );
}

#[test]
fn regression_company_slug_strips_non_alphanumeric_prefix() {
    let doc = InvoiceDocument::from_yaml_file(&fixture("valid/full-totals.yaml")).unwrap();
    assert_eq!(doc.company_slug(), "123nu");
    let filename = render_output_filename(&doc, DEFAULT_FILENAME_TEMPLATE).unwrap();
    assert_eq!(filename, "123nu-20260501-0009.html");
}

#[test]
fn regression_cli_help_shows_usage() {
    Command::cargo_bin("invoicemd-cli")
        .expect("binary should exist for integration tests")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice YAML files"))
        .stdout(predicate::str::contains("--template"))
        .stdout(predicate::str::contains("--output-name"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn regression_cli_version_succeeds() {
    Command::cargo_bin("invoicemd-cli")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoicemd-cli"))
        .stdout(predicate::str::contains(invoicemd_cli::version::VERSION));
}

#[test]
fn regression_cli_missing_inputs_fails() {
    Command::cargo_bin("invoicemd-cli")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn regression_output_written_next_to_yaml_without_output_dir() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/minimal.yaml"), &yaml).unwrap();

    run(Cli {
        template: None,
        output_dir: None,
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(dir.path().join("betal-20260115-0042.html").is_file());
}

fn write_minimal_yaml(path: &Path) {
    fs::copy(fixture("valid/minimal.yaml"), path).unwrap();
}

#[test]
fn regression_computed_tax_appears_in_rendered_html() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    write_minimal_yaml(&yaml);

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    let html = fs::read_to_string(out.join("betal-20260115-0042.html")).unwrap();
    assert!(html.contains("USD 450.00"), "expected subtotal in HTML");
    assert!(html.contains("USD 37.125000"), "expected tax in HTML");
    assert!(html.contains("USD 487.125000"), "expected total in HTML");
}

fn assert_pdf_magic(path: &Path) {
    let bytes = fs::read(path).unwrap();
    assert!(
        bytes.starts_with(b"%PDF"),
        "expected PDF header in {} ({} bytes)",
        path.display(),
        bytes.len()
    );
    assert!(bytes.len() > 100, "expected a non-trivial PDF");
}

#[test]
fn regression_pdf_format_writes_pdf_not_html() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/minimal.yaml"), &yaml).unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Pdf],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    let pdf_path = out.join("betal-20260115-0042.pdf");
    assert!(pdf_path.is_file(), "missing {}", pdf_path.display());
    assert_pdf_magic(&pdf_path);
    assert!(!out.join("betal-20260115-0042.html").exists());
}

#[test]
fn regression_html_and_pdf_formats_write_both() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/minimal.yaml"), &yaml).unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Html, OutputFormat::Pdf],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(out.join("betal-20260115-0042.html").is_file());
    assert_pdf_magic(&out.join("betal-20260115-0042.pdf"));
}

#[test]
fn regression_pdf_uses_yaml_filename_with_pdf_extension() {
    let dir = tempdir().unwrap();
    let yaml = dir.path().join("invoice.yaml");
    fs::copy(fixture("valid/custom-output.yaml"), &yaml).unwrap();

    let out = dir.path().join("out");
    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Pdf],
        inputs: vec![yaml.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert_pdf_magic(&out.join("custo-custom-12.pdf"));
    assert!(!out.join("custo-custom-12.html").exists());
}

#[test]
fn regression_examples_acme_invoice_pdf_output() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out");

    run(Cli {
        template: None,
        output_dir: Some(out.clone()),
        output_name: None,
        format: vec![OutputFormat::Pdf],
        inputs: vec![examples_dir()
            .join("acme-invoice.yaml")
            .to_string_lossy()
            .into_owned()],
    })
    .expect("acme example should render to PDF");

    assert_pdf_magic(&out.join("acmec-20260315-1042.pdf"));
}
