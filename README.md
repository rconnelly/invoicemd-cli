# invoicemd-cli

Generate human-readable HTML invoices from YAML data files using [Tera](https://keats.github.io/tera/) templates.

## Features

- Render one or many invoices from `.yaml` / `.yml` files
- Accept a single file, a glob pattern, or a directory (recursive)
- Bundled default HTML invoice template (custom templates supported)
- Validate invoice YAML structure and arithmetic consistency
- Configurable output filenames via Tera template strings
- Built-in `--help` usage documentation

## Installation

```bash
cargo build --release
```

The binary is written to `target/release/invoicemd-cli`.

## Quick start

```bash
# Generate HTML for one invoice using the bundled default template
cargo run -- examples/acme-invoice.yaml

# Write all example invoices to ./output
cargo run -- -d output examples/

# Use a custom HTML template
cargo run -- -t templates/default.html -d output examples/*.yaml
```

Each input YAML file produces one HTML file. By default, output is written next to the input file unless `-d/--output-dir` is set.

## CLI usage

```
invoicemd-cli [OPTIONS] <INPUT>...

Arguments:
  <INPUT>...  One or more YAML inputs: file path, glob pattern, or directory

Options:
  -t, --template <TEMPLATE>    HTML template file (bundled default when omitted)
  -d, --output-dir <OUTPUT_DIR>  Directory for generated HTML files
      --output-name <TEMPLATE>   Global output filename template (overrides YAML and default)
  -h, --help                     Print help
  -V, --version                  Print version
```

Show help:

```bash
cargo run -- --help
```

### Input modes

| Input | Example | Behavior |
| --- | --- | --- |
| Single file | `invoices/march.yaml` | Processes that file |
| Glob | `invoices/*.yaml` | Processes all matching YAML files |
| Directory | `invoices/` | Recursively finds `.yaml` and `.yml` files |

## Invoice YAML schema

Each invoice file is a single YAML document with the following structure.

### Required fields

| Field | Type | Description |
| --- | --- | --- |
| `invoice.number` | integer | Invoice number (used in output filename) |
| `invoice.date` | date (`YYYY-MM-DD`) | Invoice issue date |
| `company.name` | string | Seller / issuer name |
| `company.address` | string | Seller address (multi-line allowed) |
| `client.name` | string | Bill-to customer name |
| `client.address` | string | Customer address |
| `line_items` | list (min 1) | Billable line items |

Each line item requires:

| Field | Type | Description |
| --- | --- | --- |
| `description` | string | Line item description |
| `quantity` | decimal | Must be > 0 |
| `unit_price` | decimal | Must be >= 0 |

### Optional fields

| Field | Type | Description |
| --- | --- | --- |
| `invoice.due_date` | date | Must be on or after `invoice.date` |
| `invoice.currency` | string | Currency code (default: `USD`) |
| `invoice.payment_terms` | string | e.g. `Net 30` |
| `company.email` | string | Seller email |
| `company.phone` | string | Seller phone |
| `company.tax_id` | string | Tax / VAT ID |
| `client.email` | string | Client email |
| `client.phone` | string | Client phone |
| `line_items[].amount` | decimal | If set, must equal `quantity * unit_price` |
| `totals.subtotal` | decimal | If set, must match sum of line items |
| `totals.tax_rate` | decimal | Tax rate applied to subtotal (e.g. `0.0825`) |
| `totals.tax` | decimal | If set, must match computed tax |
| `totals.total` | decimal | If set, must match subtotal + tax |
| `notes` | list of strings | Footer notes |
| `output.filename` | string | Per-invoice output filename Tera template |

### Example

See [`examples/acme-invoice.yaml`](examples/acme-invoice.yaml).

```yaml
invoice:
  number: 1042
  date: 2026-03-15
  currency: USD

company:
  name: Acme Corporation
  address: |
    123 Market Street
    San Francisco, CA 94105

client:
  name: Example Client LLC
  address: 456 Pine Avenue

line_items:
  - description: Consulting
    quantity: "8"
    unit_price: "175.00"

totals:
  tax_rate: "0.0825"
```

Decimals may be written as strings or numbers. Omitted totals are computed automatically from line items and `tax_rate`.

## HTML templates

Templates are [Tera](https://keats.github.io/tera/) HTML files. Pass a custom template with `-t/--template`; otherwise the bundled [`templates/default.html`](templates/default.html) is used.

### Template context

These variables are available in both HTML and filename templates:

| Variable | Description |
| --- | --- |
| `invoice` | `number`, `date`, `due_date`, `currency`, `payment_terms` |
| `company` | Seller party (`name`, `address`, `email`, `phone`, `tax_id`) |
| `client` | Bill-to party |
| `line_items` | List with `description`, `quantity`, `unit_price`, `amount` |
| `totals` | `subtotal`, `tax_rate`, `tax`, `total`, `has_tax` |
| `notes` | List of note strings |
| `company_slug` | First 5 alphanumeric characters of company name, lowercased |
| `invoice_date` | Issue date as `YYYYMMDD` |
| `invoice_number` | Zero-padded invoice number (4 digits) |

Tera filters such as `slugify`, `replace`, and `safe` work in custom templates.

## Output filenames

**Default pattern:**

```
[first 5 alnum chars of company name]-[yyyymmdd]-[zero-padded invoice number].html
```

Example: `acmec-20260315-1042.html` for Acme Corporation, invoice #1042, dated 2026-03-15.

**Override priority (highest first):**

1. `--output-name` CLI flag
2. `output.filename` in the YAML file
3. Built-in default template

Example custom filename in YAML:

```yaml
output:
  filename: "{{ company.name | slugify }}-inv-{{ invoice.number }}.html"
```

Rendered filenames must not contain path separators (`/` or `\`).

## Development

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## License

MIT
