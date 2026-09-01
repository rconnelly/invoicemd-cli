use anyhow::{bail, Context, Result};
use printpdf::{GeneratePdfOptions, PdfDocument, PdfSaveOptions};
use std::collections::BTreeMap;

/// Convert rendered invoice HTML into a PDF document (A4).
pub fn html_to_pdf(html: &str) -> Result<Vec<u8>> {
    let document = html_to_pdf_document(html)?;
    let mut warnings = Vec::new();
    Ok(document.save(&PdfSaveOptions::default(), &mut warnings))
}

fn html_to_pdf_document(html: &str) -> Result<PdfDocument> {
    let html = prepare_html(html);
    let images = BTreeMap::new();
    let fonts = BTreeMap::new();
    let options = GeneratePdfOptions {
        page_width: Some(210.0),
        page_height: Some(297.0),
        font_embedding: Some(true),
        ..Default::default()
    };
    let mut warnings = Vec::new();
    let document = PdfDocument::from_html(&html, &images, &fonts, &options, &mut warnings)
        .map_err(|err| anyhow::anyhow!(err))
        .context("failed to render invoice PDF from HTML")?;

    if document.pages.is_empty() {
        bail!("PDF renderer produced no pages");
    }

    Ok(document)
}

fn prepare_html(html: &str) -> String {
    let trimmed = html.trim_start();
    let without_doctype = match trimmed.split_once('>') {
        Some((first, rest)) if first.eq_ignore_ascii_case("<!doctype html") => rest.trim_start(),
        _ => trimmed,
    };
    without_doctype.replace("<br>", "<br />")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::InvoiceDocument;
    use crate::render::{build_renderer, render_invoice_html};
    use std::path::PathBuf;

    fn extracted_text(document: &PdfDocument) -> String {
        document
            .extract_text()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn prepare_html_strips_doctype_and_closes_br() {
        let html = "<!DOCTYPE html>\n<html><body>A<br>B</body></html>";
        assert_eq!(prepare_html(html), "<html><body>A<br />B</body></html>");
    }

    #[test]
    fn converts_simple_html_to_pdf() {
        let pdf = html_to_pdf("<html><body><p>Invoice Test</p></body></html>").unwrap();
        assert!(
            pdf.starts_with(b"%PDF"),
            "expected PDF magic bytes, got {} bytes",
            pdf.len()
        );
        assert!(pdf.len() > 100, "expected a non-trivial PDF");
    }

    #[test]
    fn invoice_html_pdf_contains_company_and_number() {
        let yaml =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/minimal.yaml");
        let doc = InvoiceDocument::from_yaml_file(&yaml).unwrap();
        let renderer = build_renderer(None).unwrap();
        let html = render_invoice_html(&renderer, &doc).unwrap();
        let pdf_doc = html_to_pdf_document(&html).unwrap();
        let text = extracted_text(&pdf_doc);
        assert!(
            text.contains("Beta Labs LLC"),
            "expected company name in PDF text, got: {text}"
        );
        assert!(
            text.contains("42"),
            "expected invoice number in PDF text, got: {text}"
        );
    }
}
