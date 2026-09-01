//! Structural validation of invoice YAML against `schema/invoice.schema.yaml`.
//!
//! Arithmetic checks (line amounts, totals, due_date vs date) stay in the CLI.

use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn yaml_to_json(yaml: &str) -> Value {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).expect("YAML should parse");
    serde_json::to_value(value).expect("YAML value should convert to JSON")
}

fn compile_invoice_schema() -> Validator {
    let schema = yaml_to_json(include_str!("../schema/invoice.schema.yaml"));
    jsonschema::draft7::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("invoice JSON Schema should compile")
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn yaml_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml" | "yml")
            )
        })
        .collect();
    files.sort();
    files
}

fn error_report(validator: &Validator, instance: &Value) -> String {
    validator.iter_errors(instance).into_errors().to_string()
}

fn assert_schema_valid(validator: &Validator, path: &Path) {
    let yaml = fs::read_to_string(path).unwrap();
    let instance = yaml_to_json(&yaml);
    if !validator.is_valid(&instance) {
        panic!(
            "{} should satisfy the JSON Schema:\n{}",
            path.display(),
            error_report(validator, &instance)
        );
    }
}

fn assert_schema_invalid(validator: &Validator, path: &Path) {
    let yaml = fs::read_to_string(path).unwrap();
    let instance = yaml_to_json(&yaml);
    assert!(
        !validator.is_valid(&instance),
        "{} should be rejected by the JSON Schema",
        path.display()
    );
}

#[test]
fn schema_compiles_and_matches_meta_schema() {
    let schema = yaml_to_json(include_str!("../schema/invoice.schema.yaml"));
    assert!(
        jsonschema::draft7::meta::is_valid(&schema),
        "invoice schema should be a valid Draft 7 document"
    );
    let _ = compile_invoice_schema();
}

#[test]
fn examples_satisfy_schema() {
    let validator = compile_invoice_schema();
    let files = yaml_files_in(&crate_root().join("examples"));
    assert!(!files.is_empty(), "expected example YAML files");
    for path in files {
        assert_schema_valid(&validator, &path);
    }
}

#[test]
fn valid_fixtures_satisfy_schema() {
    let validator = compile_invoice_schema();
    let files = yaml_files_in(&crate_root().join("tests/fixtures/valid"));
    assert!(!files.is_empty(), "expected valid fixtures");
    for path in files {
        assert_schema_valid(&validator, &path);
    }
}

#[test]
fn empty_line_items_fixture_fails_schema() {
    let validator = compile_invoice_schema();
    assert_schema_invalid(
        &validator,
        &crate_root().join("tests/fixtures/invalid/empty-line-items.yaml"),
    );
}

#[test]
fn arithmetic_invalid_fixtures_are_structurally_valid() {
    // These documents are well-formed YAML; the CLI rejects them for math/date rules
    // that JSON Schema cannot express.
    let validator = compile_invoice_schema();
    for file in [
        "invalid/due-before-issue.yaml",
        "invalid/mismatched-subtotal.yaml",
        "invalid/mismatched-line-amount.yaml",
    ] {
        assert_schema_valid(&validator, &crate_root().join("tests/fixtures").join(file));
    }
}

#[test]
fn schema_rejects_unknown_properties_and_blank_names() {
    let validator = compile_invoice_schema();

    let missing_company = r#"
invoice:
  number: 1
  date: 2026-01-01
client:
  name: Client
  address: 2 Main
line_items:
  - description: Item
    quantity: "1"
    unit_price: "10.00"
"#;
    assert!(!validator.is_valid(&yaml_to_json(missing_company)));

    let blank_name = r#"
invoice:
  number: 1
  date: 2026-01-01
company:
  name: "   "
  address: 1 Main
client:
  name: Client
  address: 2 Main
line_items:
  - description: Item
    quantity: "1"
    unit_price: "10.00"
"#;
    assert!(!validator.is_valid(&yaml_to_json(blank_name)));

    let zero_quantity = r#"
invoice:
  number: 1
  date: 2026-01-01
company:
  name: Acme
  address: 1 Main
client:
  name: Client
  address: 2 Main
line_items:
  - description: Item
    quantity: "0"
    unit_price: "10.00"
"#;
    assert!(!validator.is_valid(&yaml_to_json(zero_quantity)));

    let unknown_field = r#"
invoice:
  number: 1
  date: 2026-01-01
  nickname: extra
company:
  name: Acme
  address: 1 Main
client:
  name: Client
  address: 2 Main
line_items:
  - description: Item
    quantity: "1"
    unit_price: "10.00"
"#;
    assert!(!validator.is_valid(&yaml_to_json(unknown_field)));

    let path_in_filename = r#"
invoice:
  number: 1
  date: 2026-01-01
company:
  name: Acme
  address: 1 Main
client:
  name: Client
  address: 2 Main
line_items:
  - description: Item
    quantity: "1"
    unit_price: "10.00"
output:
  filename: "subdir/invoice.html"
"#;
    assert!(!validator.is_valid(&yaml_to_json(path_in_filename)));
}
