mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tokscale")]
#[command(author, version, about = "AI token usage analytics")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "blue")]
    theme: String,

    #[arg(short, long, default_value = "0")]
    refresh: u64,

    #[arg(long)]
    debug: bool,

    #[arg(long)]
    test_data: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Show model usage report")]
    Models {
        #[arg(long)]
        json: bool,
    },
    #[command(about = "Show monthly usage report")]
    Monthly {
        #[arg(long)]
        json: bool,
    },
    #[command(about = "Show pricing for a model")]
    Pricing { model_id: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.test_data {
        return tui::test_data_loading();
    }

    match cli.command {
        Some(Commands::Models { json }) => {
            run_models_report(json)
        }
        Some(Commands::Monthly { json }) => {
            run_monthly_report(json)
        }
        Some(Commands::Pricing { model_id }) => {
            run_pricing_lookup(&model_id)
        }
        None => {
            tui::run(&cli.theme, cli.refresh, cli.debug)
        }
    }
}

fn run_models_report(json: bool) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_model_report, ReportOptions};

    let rt = Runtime::new()?;
    let report = rt.block_on(async {
        get_model_report(ReportOptions {
            home_dir: None,
            sources: None,
            since: None,
            until: None,
            year: None,
        })
        .await
    }).map_err(|e| anyhow::anyhow!(e))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        use comfy_table::{Table, ContentArrangement};

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["Source", "Model", "Input", "Output", "Cache", "Cost"]);

        for entry in &report.entries {
            table.add_row(vec![
                entry.source.clone(),
                entry.model.clone(),
                format_tokens(entry.input),
                format_tokens(entry.output),
                format_tokens(entry.cache_read),
                format_currency(entry.cost),
            ]);
        }

        println!("{table}");
        println!("\nTotal: {} | Cost: {}", 
            format_tokens(report.total_input + report.total_output + report.total_cache_read),
            format_currency(report.total_cost)
        );
    }

    Ok(())
}

fn run_monthly_report(json: bool) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_monthly_report, ReportOptions};

    let rt = Runtime::new()?;
    let report = rt.block_on(async {
        get_monthly_report(ReportOptions {
            home_dir: None,
            sources: None,
            since: None,
            until: None,
            year: None,
        })
        .await
    }).map_err(|e| anyhow::anyhow!(e))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        use comfy_table::{Table, ContentArrangement};

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["Month", "Models", "Input", "Output", "Cost"]);

        for entry in &report.entries {
            table.add_row(vec![
                entry.month.clone(),
                entry.models.len().to_string(),
                format_tokens(entry.input),
                format_tokens(entry.output),
                format_currency(entry.cost),
            ]);
        }

        println!("{table}");
        println!("\nTotal Cost: {}", format_currency(report.total_cost));
    }

    Ok(())
}

fn run_pricing_lookup(model_id: &str) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::pricing::PricingService;

    let rt = Runtime::new()?;
    let result = rt.block_on(async {
        let svc = PricingService::get_or_init().await?;
        Ok::<_, String>(svc.lookup_with_source(model_id, None))
    }).map_err(|e| anyhow::anyhow!(e))?;

    match result {
        Some(pricing) => {
            println!("Model: {}", model_id);
            println!("Matched: {}", pricing.matched_key);
            println!("Source: {}", pricing.source);
            if let Some(input) = pricing.pricing.input_cost_per_token {
                println!("Input: ${:.6}/token (${:.2}/1M)", input, input * 1_000_000.0);
            }
            if let Some(output) = pricing.pricing.output_cost_per_token {
                println!("Output: ${:.6}/token (${:.2}/1M)", output, output * 1_000_000.0);
            }
        }
        None => {
            println!("Model not found: {}", model_id);
        }
    }

    Ok(())
}

fn format_tokens(n: i64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_currency(n: f64) -> String {
    if n >= 1000.0 {
        format!("${:.2}K", n / 1000.0)
    } else {
        format!("${:.2}", n)
    }
}
