use crate::version;
use clap::{Parser, ValueEnum, ValueHint};
use std::path::PathBuf;

/// Generate human-readable HTML and PDF invoices from YAML data files.
#[derive(Parser, Debug)]
#[command(
    name = "invoicemd-cli",
    version = version::VERSION,
    long_version = version::LONG_VERSION,
    about = "Generate HTML and PDF invoices from YAML using Tera templates",
    long_about = "invoicemd-cli reads one or more invoice YAML files and renders each one \
to a standalone HTML and/or PDF file using a Tera HTML template.\n\n\
INPUT can be:\n  \
• a single .yaml/.yml file\n  \
• a glob pattern (e.g. invoices/*.yaml)\n  \
• a directory (all .yaml/.yml files inside, recursively)\n\n\
Output filenames default to:\n  \
  [first 5 alnum chars of company name]-[yyyymmdd]-[zero-padded invoice number].html\n\n\
PDF output uses the same rendered HTML (including custom --template files). The file \
extension is set from --format.\n\n\
You can override the filename per invoice in YAML under output.filename, or globally \
with --output-name. Filename templates use the same Tera variables as HTML templates \
(company_slug, invoice_date, invoice_number, invoice, company, client, etc.)."
)]
pub struct Cli {
    /// HTML template file. Uses the bundled default invoice template when omitted.
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub template: Option<PathBuf>,

    /// Directory to write generated files into. Defaults to each input file's directory.
    #[arg(short = 'd', long, value_hint = ValueHint::DirPath)]
    pub output_dir: Option<PathBuf>,

    /// Global output filename template (overrides YAML output.filename and the built-in default).
    #[arg(long)]
    pub output_name: Option<String>,

    /// Output format(s). Comma-separate to emit more than one file (`html,pdf`).
    #[arg(
        short = 'f',
        long,
        value_enum,
        default_value = "html",
        value_delimiter = ','
    )]
    pub format: Vec<OutputFormat>,

    /// One or more YAML inputs: file path, glob pattern, or directory.
    #[arg(required = true, value_hint = ValueHint::AnyPath)]
    pub inputs: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Html,
    Pdf,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Pdf => "pdf",
        }
    }
}

pub fn unique_formats(formats: &[OutputFormat]) -> Vec<OutputFormat> {
    let mut unique = Vec::new();
    for format in formats {
        if !unique.contains(format) {
            unique.push(*format);
        }
    }
    if unique.is_empty() {
        unique.push(OutputFormat::Html);
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_formats_dedupes_and_keeps_order() {
        assert_eq!(
            unique_formats(&[OutputFormat::Pdf, OutputFormat::Html, OutputFormat::Pdf]),
            vec![OutputFormat::Pdf, OutputFormat::Html]
        );
    }

    #[test]
    fn unique_formats_defaults_empty_to_html() {
        assert_eq!(unique_formats(&[]), vec![OutputFormat::Html]);
    }
}
