use crate::version;
use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// Generate human-readable HTML invoices from YAML data files.
#[derive(Parser, Debug)]
#[command(
    name = "invoicemd-cli",
    version = version::VERSION,
    long_version = version::LONG_VERSION,
    about = "Generate HTML invoices from YAML using Tera templates",
    long_about = "invoicemd-cli reads one or more invoice YAML files and renders each one \
to a standalone HTML file using a Tera HTML template.\n\n\
INPUT can be:\n  \
• a single .yaml/.yml file\n  \
• a glob pattern (e.g. invoices/*.yaml)\n  \
• a directory (all .yaml/.yml files inside, recursively)\n\n\
Output filenames default to:\n  \
  [first 5 alnum chars of company name]-[yyyymmdd]-[zero-padded invoice number].html\n\n\
You can override the filename per invoice in YAML under output.filename, or globally \
with --output-name. Filename templates use the same Tera variables as HTML templates \
(company_slug, invoice_date, invoice_number, invoice, company, client, etc.)."
)]
pub struct Cli {
    /// HTML template file. Uses the bundled default invoice template when omitted.
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub template: Option<PathBuf>,

    /// Directory to write generated HTML files into. Defaults to each input file's directory.
    #[arg(short = 'd', long, value_hint = ValueHint::DirPath)]
    pub output_dir: Option<PathBuf>,

    /// Global output filename template (overrides YAML output.filename and the built-in default).
    #[arg(long)]
    pub output_name: Option<String>,

    /// One or more YAML inputs: file path, glob pattern, or directory.
    #[arg(required = true, value_hint = ValueHint::AnyPath)]
    pub inputs: Vec<String>,
}
