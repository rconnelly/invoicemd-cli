use crate::invoice::InvoiceDocument;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

pub const DEFAULT_HTML_TEMPLATE: &str = include_str!("../templates/default.html");
pub const DEFAULT_FILENAME_TEMPLATE: &str =
    "{{ company_slug }}-{{ invoice_date }}-{{ invoice_number }}.html";

#[derive(Serialize)]
struct InvoiceTemplateContext<'a> {
    invoice: InvoiceView<'a>,
    company: PartyView<'a>,
    client: PartyView<'a>,
    line_items: Vec<LineItemView<'a>>,
    totals: TotalsView,
    notes: &'a [String],
    company_slug: String,
    invoice_date: String,
    invoice_number: String,
}

#[derive(Serialize)]
struct InvoiceView<'a> {
    number: u64,
    date: NaiveDate,
    due_date: Option<NaiveDate>,
    currency: &'a str,
    payment_terms: Option<&'a str>,
}

#[derive(Serialize)]
struct PartyView<'a> {
    name: &'a str,
    address: &'a str,
    email: Option<&'a str>,
    phone: Option<&'a str>,
    tax_id: Option<&'a str>,
}

#[derive(Serialize)]
struct LineItemView<'a> {
    description: &'a str,
    quantity: Decimal,
    unit_price: Decimal,
    amount: Decimal,
}

#[derive(Serialize)]
struct TotalsView {
    subtotal: Decimal,
    tax_rate: Option<Decimal>,
    tax: Decimal,
    total: Decimal,
    has_tax: bool,
}

pub struct Renderer {
    tera: Tera,
    template_name: String,
}

pub fn build_renderer(template_path: Option<&Path>) -> Result<Renderer> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![]);

    let template_name = if let Some(path) = template_path {
        let name = "invoice.html".to_string();
        let template_source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read template file {}", path.display()))?;
        tera.add_raw_template(&name, &template_source)
            .with_context(|| format!("failed to parse template file {}", path.display()))?;
        name
    } else {
        let name = "default.html".to_string();
        tera.add_raw_template(&name, DEFAULT_HTML_TEMPLATE)
            .context("failed to parse bundled default template")?;
        name
    };

    Ok(Renderer {
        tera,
        template_name,
    })
}

pub fn render_invoice_html(renderer: &Renderer, doc: &InvoiceDocument) -> Result<String> {
    let context = build_context(doc);
    let tera_context = context_to_tera(&context)?;
    renderer
        .tera
        .render(&renderer.template_name, &tera_context)
        .context("failed to render invoice HTML")
}

pub fn render_output_filename(doc: &InvoiceDocument, filename_template: &str) -> Result<String> {
    let context = build_context(doc);
    let tera_context = context_to_tera(&context)?;
    let rendered = Tera::one_off(filename_template, &tera_context, false)
        .with_context(|| format!("invalid output filename template '{filename_template}'"))?;

    sanitize_filename(&rendered)
}

fn build_context(doc: &InvoiceDocument) -> InvoiceTemplateContext<'_> {
    let subtotal = doc.computed_subtotal();
    let tax = doc.computed_tax(subtotal);
    let total = doc.computed_total(subtotal, tax);
    let currency = doc.invoice.currency.as_deref().unwrap_or("USD");

    InvoiceTemplateContext {
        invoice: InvoiceView {
            number: doc.invoice.number,
            date: doc.invoice.date,
            due_date: doc.invoice.due_date,
            currency,
            payment_terms: doc.invoice.payment_terms.as_deref(),
        },
        company: party_view(&doc.company),
        client: party_view(&doc.client),
        line_items: doc
            .line_items
            .iter()
            .map(|item| LineItemView {
                description: &item.description,
                quantity: item.quantity,
                unit_price: item.unit_price,
                amount: item
                    .amount
                    .unwrap_or_else(|| item.quantity * item.unit_price),
            })
            .collect(),
        totals: TotalsView {
            subtotal,
            tax_rate: doc.totals.tax_rate,
            tax,
            total,
            has_tax: tax > Decimal::ZERO,
        },
        notes: &doc.notes,
        company_slug: doc.company_slug(),
        invoice_date: doc.invoice_date_compact(),
        invoice_number: doc.padded_invoice_number(4),
    }
}

fn party_view(party: &crate::invoice::Party) -> PartyView<'_> {
    PartyView {
        name: &party.name,
        address: &party.address,
        email: party.email.as_deref(),
        phone: party.phone.as_deref(),
        tax_id: party.tax_id.as_deref(),
    }
}

fn context_to_tera(context: &InvoiceTemplateContext<'_>) -> Result<TeraContext> {
    TeraContext::from_serialize(context).context("failed to build Tera context")
}

fn sanitize_filename(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("rendered output filename is empty");
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        anyhow::bail!("rendered output filename must not contain path separators: {trimmed}");
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::{InvoiceDocument, InvoiceMeta, LineItem, OutputOptions, Party, Totals};
    use chrono::NaiveDate;
    use std::str::FromStr;

    fn dec(value: &str) -> Decimal {
        Decimal::from_str(value).unwrap()
    }

    fn sample_doc() -> InvoiceDocument {
        InvoiceDocument {
            invoice: InvoiceMeta {
                number: 7,
                date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
                due_date: None,
                currency: Some("USD".into()),
                payment_terms: None,
            },
            company: Party {
                name: "Acme Corporation".into(),
                address: "123 Main St".into(),
                email: None,
                phone: None,
                tax_id: None,
            },
            client: Party {
                name: "Client Co".into(),
                address: "456 Oak Ave".into(),
                email: None,
                phone: None,
                tax_id: None,
            },
            line_items: vec![LineItem {
                description: "Consulting".into(),
                quantity: dec("2"),
                unit_price: dec("100"),
                amount: None,
            }],
            totals: Totals::default(),
            notes: vec![],
            output: OutputOptions::default(),
        }
    }

    #[test]
    fn default_filename_template_matches_spec() {
        let doc = sample_doc();
        let filename = render_output_filename(&doc, DEFAULT_FILENAME_TEMPLATE).unwrap();
        assert_eq!(filename, "acmec-20260315-0007.html");
    }

    #[test]
    fn renders_default_html_template() {
        let doc = sample_doc();
        let renderer = build_renderer(None).unwrap();
        let html = render_invoice_html(&renderer, &doc).unwrap();
        assert!(html.contains("Acme Corporation"));
        assert!(html.contains("Invoice #7"));
    }
}
