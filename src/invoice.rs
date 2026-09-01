use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::path::Path;

/// Root document deserialized from an invoice YAML file.
#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceDocument {
    pub invoice: InvoiceMeta,
    pub company: Party,
    pub client: Party,
    pub line_items: Vec<LineItem>,
    #[serde(default)]
    pub totals: Totals,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub output: OutputOptions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceMeta {
    pub number: u64,
    pub date: NaiveDate,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub payment_terms: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Party {
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    #[serde(default)]
    pub amount: Option<Decimal>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Totals {
    #[serde(default)]
    pub subtotal: Option<Decimal>,
    #[serde(default)]
    pub tax_rate: Option<Decimal>,
    #[serde(default)]
    pub tax: Option<Decimal>,
    #[serde(default)]
    pub total: Option<Decimal>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct OutputOptions {
    /// Tera template string for the output HTML filename.
    /// Example: `{{ company_slug }}-{{ invoice_date }}-{{ invoice_number }}.html`
    #[serde(default)]
    pub filename: Option<String>,
}

impl InvoiceDocument {
    pub fn from_yaml_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read YAML file {}", path.display()))?;
        let doc: InvoiceDocument = serde_yaml::from_str(&contents)
            .with_context(|| format!("failed to parse YAML file {}", path.display()))?;
        doc.validate()?;
        Ok(doc)
    }

    pub fn validate(&self) -> Result<()> {
        if self.company.name.trim().is_empty() {
            bail!("company.name is required and cannot be blank");
        }
        if self.client.name.trim().is_empty() {
            bail!("client.name is required and cannot be blank");
        }
        if self.company.address.trim().is_empty() {
            bail!("company.address is required and cannot be blank");
        }
        if self.client.address.trim().is_empty() {
            bail!("client.address is required and cannot be blank");
        }
        if self.line_items.is_empty() {
            bail!("line_items must contain at least one entry");
        }

        for (index, item) in self.line_items.iter().enumerate() {
            if item.description.trim().is_empty() {
                bail!("line_items[{index}].description cannot be blank");
            }
            if item.quantity <= Decimal::ZERO {
                bail!("line_items[{index}].quantity must be greater than zero");
            }
            if item.unit_price < Decimal::ZERO {
                bail!("line_items[{index}].unit_price cannot be negative");
            }
            if let Some(amount) = item.amount {
                let expected = item.quantity * item.unit_price;
                if amount != expected {
                    bail!(
                        "line_items[{index}].amount ({amount}) does not match quantity * unit_price ({expected})"
                    );
                }
            }
        }

        if let Some(due_date) = self.invoice.due_date {
            if due_date < self.invoice.date {
                bail!("invoice.due_date cannot be before invoice.date");
            }
        }

        let computed_subtotal = self.computed_subtotal();
        if let Some(subtotal) = self.totals.subtotal {
            if subtotal != computed_subtotal {
                bail!(
                    "totals.subtotal ({subtotal}) does not match sum of line items ({computed_subtotal})"
                );
            }
        }

        let tax = self.computed_tax(computed_subtotal);
        if let Some(declared_tax) = self.totals.tax {
            if declared_tax != tax {
                bail!("totals.tax ({declared_tax}) does not match computed tax ({tax})");
            }
        }

        let total = self.computed_total(computed_subtotal, tax);
        if let Some(declared_total) = self.totals.total {
            if declared_total != total {
                bail!("totals.total ({declared_total}) does not match computed total ({total})");
            }
        }

        Ok(())
    }

    pub fn computed_subtotal(&self) -> Decimal {
        self.line_items
            .iter()
            .map(|item| {
                item.amount
                    .unwrap_or_else(|| item.quantity * item.unit_price)
            })
            .sum()
    }

    pub fn computed_tax(&self, subtotal: Decimal) -> Decimal {
        if let Some(tax) = self.totals.tax {
            return tax;
        }
        self.totals
            .tax_rate
            .map(|rate| subtotal * rate)
            .unwrap_or(Decimal::ZERO)
    }

    pub fn computed_total(&self, subtotal: Decimal, tax: Decimal) -> Decimal {
        if let Some(total) = self.totals.total {
            return total;
        }
        subtotal + tax
    }

    /// First five alphanumeric characters from the company name, lowercased.
    pub fn company_slug(&self) -> String {
        self.company
            .name
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .take(5)
            .collect::<String>()
            .to_ascii_lowercase()
    }

    pub fn invoice_date_compact(&self) -> String {
        self.invoice.date.format("%Y%m%d").to_string()
    }

    pub fn padded_invoice_number(&self, width: usize) -> String {
        format!("{:0width$}", self.invoice.number, width = width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn validates_happy_path() {
        sample_doc().validate().unwrap();
    }

    #[test]
    fn company_slug_is_first_five_alnum_chars() {
        let doc = sample_doc();
        assert_eq!(doc.company_slug(), "acmec");
    }

    #[test]
    fn rejects_empty_line_items() {
        let mut doc = sample_doc();
        doc.line_items.clear();
        let err = doc.validate().unwrap_err();
        assert!(format!("{err:#}").contains("line_items"));
    }

    #[test]
    fn rejects_blank_company_name() {
        let mut doc = sample_doc();
        doc.company.name = "   ".into();
        let err = doc.validate().unwrap_err();
        assert!(format!("{err:#}").contains("company.name"));
    }

    #[test]
    fn rejects_due_date_before_issue_date() {
        let mut doc = sample_doc();
        doc.invoice.due_date = Some(NaiveDate::from_ymd_opt(2026, 3, 14).unwrap());
        let err = doc.validate().unwrap_err();
        assert!(format!("{err:#}").contains("due_date"));
    }

    #[test]
    fn rejects_mismatched_line_item_amount() {
        let mut doc = sample_doc();
        doc.line_items[0].amount = Some(dec("999"));
        let err = doc.validate().unwrap_err();
        assert!(format!("{err:#}").contains("line_items[0].amount"));
    }

    #[test]
    fn rejects_zero_quantity() {
        let mut doc = sample_doc();
        doc.line_items[0].quantity = dec("0");
        let err = doc.validate().unwrap_err();
        assert!(format!("{err:#}").contains("quantity"));
    }

    #[test]
    fn computed_subtotal_sums_line_items() {
        let doc = sample_doc();
        assert_eq!(doc.computed_subtotal(), dec("200"));
    }

    #[test]
    fn computed_tax_applies_rate_when_tax_not_declared() {
        let mut doc = sample_doc();
        doc.totals.tax_rate = Some(dec("0.10"));
        let subtotal = doc.computed_subtotal();
        assert_eq!(doc.computed_tax(subtotal), dec("20"));
    }

    #[test]
    fn padded_invoice_number_zero_fills_to_four_digits() {
        let doc = sample_doc();
        assert_eq!(doc.padded_invoice_number(4), "0007");
        assert_eq!(doc.invoice_date_compact(), "20260315");
    }
}
