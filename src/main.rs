use clap::Parser;

fn main() -> anyhow::Result<()> {
    invoicemd_cli::run(invoicemd_cli::cli::Cli::parse())
}
