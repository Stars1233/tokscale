mod tui;
mod auth;
mod cursor;

use anyhow::Result;
use chrono::Datelike;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
        #[arg(long)]
        light: bool,
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Show only today's usage")]
        today: bool,
        #[arg(long, help = "Show last 7 days")]
        week: bool,
        #[arg(long, help = "Show current month")]
        month: bool,
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, help = "End date (YYYY-MM-DD)")]
        until: Option<String>,
        #[arg(long, help = "Filter by year (YYYY)")]
        year: Option<String>,
        #[arg(long, help = "Show processing time")]
        benchmark: bool,
        #[arg(long, help = "Disable spinner")]
        no_spinner: bool,
    },
    #[command(about = "Show monthly usage report")]
    Monthly {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        light: bool,
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Show only today's usage")]
        today: bool,
        #[arg(long, help = "Show last 7 days")]
        week: bool,
        #[arg(long, help = "Show current month")]
        month: bool,
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, help = "End date (YYYY-MM-DD)")]
        until: Option<String>,
        #[arg(long, help = "Filter by year (YYYY)")]
        year: Option<String>,
        #[arg(long, help = "Show processing time")]
        benchmark: bool,
        #[arg(long, help = "Disable spinner")]
        no_spinner: bool,
    },
    #[command(about = "Show pricing for a model")]
    Pricing {
        model_id: String,
        #[arg(long, help = "Output as JSON")]
        json: bool,
        #[arg(long, help = "Force specific provider (litellm or openrouter)")]
        provider: Option<String>,
    },
    #[command(about = "Show local scan locations and session counts")]
    Sources {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Login to Tokscale (opens browser for GitHub auth)")]
    Login,
    #[command(about = "Logout from Tokscale")]
    Logout,
    #[command(about = "Show current logged in user")]
    Whoami,
    #[command(about = "Export contribution graph data as JSON")]
    Graph {
        #[arg(long, help = "Write to file instead of stdout")]
        output: Option<String>,
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Show only today's usage")]
        today: bool,
        #[arg(long, help = "Show last 7 days")]
        week: bool,
        #[arg(long, help = "Show current month")]
        month: bool,
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, help = "End date (YYYY-MM-DD)")]
        until: Option<String>,
        #[arg(long, help = "Filter by year (YYYY)")]
        year: Option<String>,
        #[arg(long, help = "Show processing time")]
        benchmark: bool,
    },
    #[command(about = "Launch interactive TUI with optional filters")]
    Tui {
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Show only today's usage")]
        today: bool,
        #[arg(long, help = "Show last 7 days")]
        week: bool,
        #[arg(long, help = "Show current month")]
        month: bool,
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, help = "End date (YYYY-MM-DD)")]
        until: Option<String>,
        #[arg(long, help = "Filter by year (YYYY)")]
        year: Option<String>,
    },
    #[command(about = "Submit usage data to the Tokscale social platform")]
    Submit {
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Show only today's usage")]
        today: bool,
        #[arg(long, help = "Show last 7 days")]
        week: bool,
        #[arg(long, help = "Show current month")]
        month: bool,
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, help = "End date (YYYY-MM-DD)")]
        until: Option<String>,
        #[arg(long, help = "Filter by year (YYYY)")]
        year: Option<String>,
        #[arg(long, help = "Show what would be submitted without actually submitting")]
        dry_run: bool,
    },
    #[command(about = "Capture subprocess output for token usage tracking")]
    Headless {
        #[arg(help = "Source CLI (currently only 'codex' supported)")]
        source: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
        #[arg(long, help = "Override output format (json or jsonl)")]
        format: Option<String>,
        #[arg(long, help = "Write captured output to file")]
        output: Option<String>,
        #[arg(long, help = "Do not auto-add JSON output flags")]
        no_auto_flags: bool,
    },
    #[command(about = "Generate year-in-review wrapped image")]
    Wrapped {
        #[arg(long, help = "Output file path (default: tokscale-{year}-wrapped.png)")]
        output: Option<String>,
        #[arg(long, help = "Year to generate (default: current year)")]
        year: Option<String>,
        #[arg(long, help = "Show only OpenCode usage")]
        opencode: bool,
        #[arg(long, help = "Show only Claude Code usage")]
        claude: bool,
        #[arg(long, help = "Show only Codex CLI usage")]
        codex: bool,
        #[arg(long, help = "Show only Gemini CLI usage")]
        gemini: bool,
        #[arg(long, help = "Show only Cursor IDE usage")]
        cursor: bool,
        #[arg(long, help = "Show only Amp usage")]
        amp: bool,
        #[arg(long, help = "Show only Droid usage")]
        droid: bool,
        #[arg(long, help = "Show only OpenClaw usage")]
        openclaw: bool,
        #[arg(long, help = "Display total tokens in abbreviated format (e.g., 7.14B)")]
        short: bool,
        #[arg(long, help = "Show Top OpenCode Agents (default)")]
        agents: bool,
        #[arg(long, help = "Show Top Clients instead of Top OpenCode Agents")]
        clients: bool,
        #[arg(long, help = "Disable pinning of Sisyphus agents in rankings")]
        disable_pinned: bool,
        #[arg(long, help = "Disable loading spinner (for scripting)")]
        no_spinner: bool,
    },
    #[command(about = "Cursor IDE integration commands")]
    Cursor {
        #[command(subcommand)]
        subcommand: CursorSubcommand,
    },
}

#[derive(Subcommand)]
enum CursorSubcommand {
    #[command(about = "Login to Cursor (paste your session token)")]
    Login {
        #[arg(long, help = "Label for this Cursor account (e.g., work, personal)")]
        name: Option<String>,
    },
    #[command(about = "Logout from a Cursor account")]
    Logout {
        #[arg(long, help = "Account label or id")]
        name: Option<String>,
        #[arg(long, help = "Logout from all Cursor accounts")]
        all: bool,
        #[arg(long, help = "Also delete cached Cursor usage")]
        purge_cache: bool,
    },
    #[command(about = "Check Cursor authentication status")]
    Status {
        #[arg(long, help = "Account label or id")]
        name: Option<String>,
    },
    #[command(about = "List saved Cursor accounts")]
    Accounts {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Switch active Cursor account")]
    Switch {
        #[arg(help = "Account label or id")]
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.test_data {
        return tui::test_data_loading();
    }

    match cli.command {
        Some(Commands::Models {
            json,
            light: _,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark: _,
            no_spinner: _,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            run_models_report(json, sources, since, until, year)
        }
        Some(Commands::Monthly {
            json,
            light: _,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark: _,
            no_spinner: _,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            run_monthly_report(json, sources, since, until, year)
        }
        Some(Commands::Pricing { model_id, json, provider }) => {
            run_pricing_lookup(&model_id, json, provider.as_deref())
        }
        Some(Commands::Sources { json }) => {
            run_sources_command(json)
        }
        Some(Commands::Login) => {
            run_login_command()
        }
        Some(Commands::Logout) => {
            run_logout_command()
        }
        Some(Commands::Whoami) => {
            run_whoami_command()
        }
        Some(Commands::Graph {
            output,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            run_graph_command(output, sources, since, until, year, benchmark)
        }
        Some(Commands::Tui {
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            today,
            week,
            month,
            since,
            until,
            year,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            tui::run(&cli.theme, cli.refresh, cli.debug, sources, since, until, year)
        }
        Some(Commands::Submit {
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            today,
            week,
            month,
            since,
            until,
            year,
            dry_run,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            run_submit_command(sources, since, until, year, dry_run)
        }
        Some(Commands::Headless {
            source,
            args,
            format,
            output,
            no_auto_flags,
        }) => {
            run_headless_command(&source, args, format, output, no_auto_flags)
        }
        Some(Commands::Wrapped {
            output,
            year,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            short,
            agents,
            clients,
            disable_pinned,
            no_spinner: _,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw,
            });
            run_wrapped_command(output, year, sources, short, agents, clients, disable_pinned)
        }
        Some(Commands::Cursor { subcommand }) => {
            run_cursor_command(subcommand)
        }
        None => {
            tui::run(&cli.theme, cli.refresh, cli.debug, None, None, None, None)
        }
    }
}

struct SourceFlags {
    opencode: bool,
    claude: bool,
    codex: bool,
    gemini: bool,
    cursor: bool,
    amp: bool,
    droid: bool,
    openclaw: bool,
}

fn build_source_filter(flags: SourceFlags) -> Option<Vec<String>> {
    let mut sources = Vec::new();
    if flags.opencode { sources.push("opencode".to_string()); }
    if flags.claude { sources.push("claude".to_string()); }
    if flags.codex { sources.push("codex".to_string()); }
    if flags.gemini { sources.push("gemini".to_string()); }
    if flags.cursor { sources.push("cursor".to_string()); }
    if flags.amp { sources.push("amp".to_string()); }
    if flags.droid { sources.push("droid".to_string()); }
    if flags.openclaw { sources.push("openclaw".to_string()); }
    
    if sources.is_empty() {
        None
    } else {
        Some(sources)
    }
}

fn build_date_filter(
    today: bool,
    week: bool,
    month: bool,
    since: Option<String>,
    until: Option<String>,
) -> (Option<String>, Option<String>) {
    use chrono::{Local, Datelike, Duration};
    
    if today {
        let date = Local::now().format("%Y-%m-%d").to_string();
        return (Some(date.clone()), Some(date));
    }
    
    if week {
        let end = Local::now();
        let start = end - Duration::days(6);
        return (
            Some(start.format("%Y-%m-%d").to_string()),
            Some(end.format("%Y-%m-%d").to_string()),
        );
    }
    
    if month {
        let now = Local::now();
        let start = now.with_day(1).unwrap();
        return (
            Some(start.format("%Y-%m-%d").to_string()),
            Some(now.format("%Y-%m-%d").to_string()),
        );
    }
    
    (since, until)
}

fn run_models_report(
    json: bool,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_model_report, ReportOptions};

    let rt = Runtime::new()?;
    let report = rt.block_on(async {
        get_model_report(ReportOptions {
            home_dir: None,
            sources,
            since,
            until,
            year,
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

fn run_monthly_report(
    json: bool,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_monthly_report, ReportOptions};

    let rt = Runtime::new()?;
    let report = rt.block_on(async {
        get_monthly_report(ReportOptions {
            home_dir: None,
            sources,
            since,
            until,
            year,
        })
        .await
    }).map_err(|e| anyhow::anyhow!(e))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        use comfy_table::{Table, ContentArrangement};

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["Month", "Input", "Output", "Cache", "Cost"]);

        for entry in &report.entries {
            table.add_row(vec![
                entry.month.clone(),
                format_tokens(entry.input),
                format_tokens(entry.output),
                format_tokens(entry.cache_read),
                format_currency(entry.cost),
            ]);
        }

        println!("{table}");
        
        // Calculate totals from entries
        let total_input: i64 = report.entries.iter().map(|e| e.input).sum();
        let total_output: i64 = report.entries.iter().map(|e| e.output).sum();
        let total_cache_read: i64 = report.entries.iter().map(|e| e.cache_read).sum();
        
        println!("\nTotal: {} | Cost: {}", 
            format_tokens(total_input + total_output + total_cache_read),
            format_currency(report.total_cost)
        );
    }

    Ok(())
}

fn run_wrapped_command(
    output: Option<String>,
    year: Option<String>,
    sources: Option<Vec<String>>,
    short: bool,
    agents: bool,
    clients: bool,
    disable_pinned: bool,
) -> Result<()> {
    use colored::Colorize;
    use std::process::Command;
    
    // Determine year
    let year = year.unwrap_or_else(|| chrono::Local::now().year().to_string());
    
    println!("{}", "\n  Tokscale - Generate Wrapped Image\n".cyan());
    
    // Check for Node.js or Bun runtime
    let runtime = if Command::new("bun").arg("--version").output().is_ok() {
        "bun"
    } else if Command::new("node").arg("--version").output().is_ok() {
        "node"
    } else {
        eprintln!("{}", "Error: Neither bun nor node found in PATH".red());
        eprintln!("{}", "Please install Node.js or Bun to use the wrapped command".bright_black());
        eprintln!("{}", "  - Bun: https://bun.sh/".bright_black());
        eprintln!("{}", "  - Node.js: https://nodejs.org/\n".bright_black());
        std::process::exit(1);
    };
    
    // Find the TypeScript CLI directory
    let exe_path = std::env::current_exe()?;
    let repo_root = exe_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Could not find repository root"))?;
    
    let cli_dir = repo_root.join("packages/cli");
    let cli_script = cli_dir.join("dist/cli.js");
    
    if !cli_script.exists() {
        eprintln!("{}", "Error: cli.js not found".red());
        eprintln!("{}", format!("  Expected at: {:?}", cli_script).bright_black());
        eprintln!("{}", "  Please run 'bun run build' in packages/cli first\n".bright_black());
        std::process::exit(1);
    }
    
    println!("{}", "  Generating wrapped image...".bright_black());
    
    // Build command arguments
    let mut args = vec![cli_script.to_string_lossy().to_string(), "wrapped".to_string()];
    
    if let Some(ref out) = output {
        args.push("--output".to_string());
        args.push(out.clone());
    }
    
    args.push("--year".to_string());
    args.push(year.clone());
    
    if let Some(ref srcs) = sources {
        for src in srcs {
            args.push(format!("--{}", src));
        }
    }
    
    if short {
        args.push("--short".to_string());
    }
    
    if clients {
        args.push("--clients".to_string());
    } else if agents {
        args.push("--agents".to_string());
    }
    
    if disable_pinned {
        args.push("--disable-pinned".to_string());
    }
    
    // Execute the wrapped generator
    let output_result = Command::new(runtime)
        .args(&args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute wrapped generator: {}", e))?;
    
    if !output_result.status.success() {
        eprintln!("{}", "\nError generating wrapped image:".red());
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
        std::process::exit(1);
    }
    
    // Parse output to get the file path
    let output_path = String::from_utf8_lossy(&output_result.stdout).trim().to_string();
    
    if !output_path.is_empty() {
        println!("{}", format!("\n  ✓ Generated wrapped image: {}\n", output_path).green());
    } else {
        println!("{}", "\n  ✓ Wrapped image generated successfully\n".green());
    }
    
    Ok(())
}

fn run_pricing_lookup(model_id: &str, json: bool, provider: Option<&str>) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::pricing::PricingService;

    let rt = Runtime::new()?;
    let result = rt.block_on(async {
        let svc = PricingService::get_or_init().await?;
        Ok::<_, String>(svc.lookup_with_source(model_id, provider))
    }).map_err(|e| anyhow::anyhow!(e))?;

    if json {
        match result {
            Some(pricing) => {
                #[derive(serde::Serialize)]
                struct PricingOutput {
                    model: String,
                    matched_key: String,
                    source: String,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    input_cost_per_token: Option<f64>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    output_cost_per_token: Option<f64>,
                }
                
                let output = PricingOutput {
                    model: model_id.to_string(),
                    matched_key: pricing.matched_key,
                    source: pricing.source,
                    input_cost_per_token: pricing.pricing.input_cost_per_token,
                    output_cost_per_token: pricing.pricing.output_cost_per_token,
                };
                
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            None => {
                #[derive(serde::Serialize)]
                struct ErrorOutput {
                    error: String,
                }
                
                let output = ErrorOutput {
                    error: "Model not found".to_string(),
                };
                
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        }
    } else {
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

fn run_sources_command(json: bool) -> Result<()> {
    use tokscale_core::{parse_local_sources, LocalParseOptions};
    
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    
    let parsed = parse_local_sources(LocalParseOptions {
        home_dir: Some(home_dir.to_string_lossy().to_string()),
        sources: Some(vec![
            "opencode".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "amp".to_string(),
            "droid".to_string(),
            "openclaw".to_string(),
        ]),
        since: None,
        until: None,
        year: None,
    }).map_err(|e| anyhow::anyhow!(e))?;
    
    let headless_roots = get_headless_roots(&home_dir);
    let headless_codex_count = parsed.messages.iter()
        .filter(|m| m.agent.as_deref() == Some("headless") && m.source == "codex")
        .count() as i32;
    
    #[derive(serde::Serialize)]
    struct SourceRow {
        source: String,
        label: String,
        sessions_path: String,
        sessions_path_exists: bool,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        legacy_paths: Vec<LegacyPath>,
        message_count: i32,
        headless_supported: bool,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        headless_paths: Vec<HeadlessPath>,
        headless_message_count: i32,
    }
    
    #[derive(serde::Serialize)]
    struct LegacyPath {
        path: String,
        exists: bool,
    }
    
    #[derive(serde::Serialize)]
    struct HeadlessPath {
        path: String,
        exists: bool,
    }
    
    let sources = vec![
        SourceRow {
            source: "opencode".to_string(),
            label: "OpenCode".to_string(),
            sessions_path: home_dir.join(".local/share/opencode/storage/message").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".local/share/opencode/storage/message").exists(),
            legacy_paths: vec![],
            message_count: parsed.opencode_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "claude".to_string(),
            label: "Claude Code".to_string(),
            sessions_path: home_dir.join(".claude/projects").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".claude/projects").exists(),
            legacy_paths: vec![],
            message_count: parsed.claude_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "codex".to_string(),
            label: "Codex CLI".to_string(),
            sessions_path: get_codex_home(&home_dir).join("sessions").to_string_lossy().to_string(),
            sessions_path_exists: get_codex_home(&home_dir).join("sessions").exists(),
            legacy_paths: vec![],
            message_count: parsed.codex_count,
            headless_supported: true,
            headless_paths: headless_roots.iter().map(|root| {
                let path = root.join("codex");
                HeadlessPath {
                    path: path.to_string_lossy().to_string(),
                    exists: path.exists(),
                }
            }).collect(),
            headless_message_count: headless_codex_count,
        },
        SourceRow {
            source: "gemini".to_string(),
            label: "Gemini CLI".to_string(),
            sessions_path: home_dir.join(".gemini/tmp").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".gemini/tmp").exists(),
            legacy_paths: vec![],
            message_count: parsed.gemini_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "cursor".to_string(),
            label: "Cursor IDE".to_string(),
            sessions_path: home_dir.join(".config/tokscale/cursor-cache").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".config/tokscale/cursor-cache").exists(),
            legacy_paths: vec![],
            message_count: 0,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "amp".to_string(),
            label: "Amp".to_string(),
            sessions_path: home_dir.join(".local/share/amp/threads").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".local/share/amp/threads").exists(),
            legacy_paths: vec![],
            message_count: parsed.amp_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "droid".to_string(),
            label: "Droid".to_string(),
            sessions_path: home_dir.join(".factory/sessions").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".factory/sessions").exists(),
            legacy_paths: vec![],
            message_count: parsed.droid_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "openclaw".to_string(),
            label: "OpenClaw".to_string(),
            sessions_path: home_dir.join(".openclaw/agents").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".openclaw/agents").exists(),
            legacy_paths: vec![
                LegacyPath {
                    path: home_dir.join(".clawdbot/agents").to_string_lossy().to_string(),
                    exists: home_dir.join(".clawdbot/agents").exists(),
                },
                LegacyPath {
                    path: home_dir.join(".moltbot/agents").to_string_lossy().to_string(),
                    exists: home_dir.join(".moltbot/agents").exists(),
                },
                LegacyPath {
                    path: home_dir.join(".moldbot/agents").to_string_lossy().to_string(),
                    exists: home_dir.join(".moldbot/agents").exists(),
                },
            ],
            message_count: parsed.openclaw_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
    ];
    
    if json {
        #[derive(serde::Serialize)]
        struct Output {
            headless_roots: Vec<String>,
            sources: Vec<SourceRow>,
            note: String,
        }
        
        let output = Output {
            headless_roots: headless_roots.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            sources,
            note: "Headless capture is supported for Codex CLI only.".to_string(),
        };
        
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        use colored::Colorize;
        
        println!("\n  {}", "Local sources & session counts".cyan());
        println!("  {}", format!("Headless roots: {}", 
            headless_roots.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join(", ")
        ).bright_black());
        println!();
        
        for row in sources {
            println!("  {}", row.label.white());
            println!("  {}", format!("sessions: {}", describe_path(&row.sessions_path, row.sessions_path_exists)).bright_black());
            
            if !row.legacy_paths.is_empty() {
                let legacy_desc: Vec<String> = row.legacy_paths.iter()
                    .map(|lp| describe_path(&lp.path, lp.exists))
                    .collect();
                println!("  {}", format!("legacy: {}", legacy_desc.join(", ")).bright_black());
            }
            
            if row.headless_supported {
                let headless_desc: Vec<String> = row.headless_paths.iter()
                    .map(|hp| describe_path(&hp.path, hp.exists))
                    .collect();
                println!("  {}", format!("headless: {}", headless_desc.join(", ")).bright_black());
                println!("  {}", format!("messages: {} (headless: {})", 
                    format_number(row.message_count), 
                    format_number(row.headless_message_count)
                ).bright_black());
            } else {
                println!("  {}", format!("messages: {}", format_number(row.message_count)).bright_black());
            }
            
            println!();
        }
        
        println!("  {}", "Note: Headless capture is supported for Codex CLI only.".bright_black());
        println!();
    }
    
    Ok(())
}

fn get_headless_roots(home_dir: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    
    if let Ok(env_dir) = std::env::var("TOKSCALE_HEADLESS_DIR") {
        roots.push(PathBuf::from(env_dir));
    } else {
        roots.push(home_dir.join(".config/tokscale/headless"));
        
        #[cfg(target_os = "macos")]
        {
            roots.push(home_dir.join("Library/Application Support/tokscale/headless"));
        }
    }
    
    roots
}

fn get_codex_home(home_dir: &Path) -> PathBuf {
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        PathBuf::from(codex_home)
    } else {
        home_dir.join(".codex")
    }
}

fn describe_path(path: &str, exists: bool) -> String {
    let path_display = path.replace(&dirs::home_dir().unwrap().to_string_lossy().to_string(), "~");
    if exists {
        format!("{} ✓", path_display)
    } else {
        format!("{} ✗", path_display)
    }
}

fn format_number(n: i32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn run_login_command() -> Result<()> {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new()?;
    rt.block_on(async {
        auth::login().await
    })
}

fn run_logout_command() -> Result<()> {
    auth::logout()
}

fn run_whoami_command() -> Result<()> {
    auth::whoami()
}

fn run_graph_command(
    output: Option<String>,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    benchmark: bool,
) -> Result<()> {
    use tokscale_core::{parse_local_sources, LocalParseOptions, aggregate_by_date, generate_graph_result, UnifiedMessage, TokenBreakdown};
    use std::time::Instant;
    
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    
    let start = Instant::now();
    
    let parsed = parse_local_sources(LocalParseOptions {
        home_dir: Some(home_dir.to_string_lossy().to_string()),
        sources: sources.clone(),
        since: since.clone(),
        until: until.clone(),
        year: year.clone(),
    }).map_err(|e| anyhow::anyhow!(e))?;
    
    let unified_messages: Vec<UnifiedMessage> = parsed.messages.into_iter().map(|msg| {
        let tokens = TokenBreakdown {
            input: msg.input,
            output: msg.output,
            cache_read: msg.cache_read,
            cache_write: msg.cache_write,
            reasoning: msg.reasoning,
        };
        
        UnifiedMessage::new_with_agent(
            msg.source,
            msg.model_id,
            msg.provider_id,
            msg.session_id,
            msg.timestamp,
            tokens,
            0.0,
            msg.agent,
        )
    }).collect();
    
    let contributions = aggregate_by_date(unified_messages);
    let processing_time_ms = start.elapsed().as_millis() as u32;
    let graph_result = generate_graph_result(contributions, processing_time_ms);
    
    let json_output = serde_json::to_string_pretty(&graph_result)?;
    
    if let Some(output_path) = output {
        std::fs::write(&output_path, json_output)?;
        
        use colored::Colorize;
        eprintln!("{}", format!("✓ Graph data written to {}", output_path).green());
        eprintln!("{}", format!("  {} days, {} sources, {} models",
            graph_result.contributions.len(),
            graph_result.summary.sources.len(),
            graph_result.summary.models.len()
        ).bright_black());
        eprintln!("{}", format!("  Total: {}", format_currency(graph_result.summary.total_cost)).bright_black());
        
        if benchmark {
            eprintln!("{}", format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black());
        }
    } else {
        println!("{}", json_output);
    }
    
    Ok(())
}

#[derive(serde::Deserialize)]
struct SubmitResponse {
    success: bool,
    #[serde(rename = "submissionId")]
    submission_id: Option<String>,
    #[allow(dead_code)]
    username: Option<String>,
    metrics: Option<SubmitMetrics>,
    warnings: Option<Vec<String>>,
    error: Option<String>,
    details: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct SubmitMetrics {
    #[serde(rename = "totalTokens")]
    total_tokens: Option<i64>,
    #[serde(rename = "totalCost")]
    total_cost: Option<f64>,
    #[serde(rename = "activeDays")]
    active_days: Option<i32>,
    #[allow(dead_code)]
    sources: Option<Vec<String>>,
}

fn run_submit_command(
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    dry_run: bool,
) -> Result<()> {
    use colored::Colorize;
    use tokio::runtime::Runtime;
    use tokscale_core::{generate_graph, ReportOptions};

    let credentials = match auth::load_credentials() {
        Some(creds) => creds,
        None => {
            eprintln!("\n  {}", "Not logged in.".yellow());
            eprintln!("{}", "  Run 'tokscale login' first.\n".bright_black());
            std::process::exit(1);
        }
    };

    println!("\n  {}\n", "Tokscale - Submit Usage Data".cyan());
    println!("{}", "  Scanning local session data...".bright_black());

    let rt = Runtime::new()?;
    let graph_result = rt.block_on(async {
        generate_graph(ReportOptions {
            home_dir: None,
            sources,
            since,
            until,
            year,
        })
        .await
    }).map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", "  Data to submit:".white());
    println!("{}", format!("    Date range: {} to {}",
        graph_result.meta.date_range_start,
        graph_result.meta.date_range_end,
    ).bright_black());
    println!("{}", format!("    Active days: {}", graph_result.summary.active_days).bright_black());
    println!("{}", format!("    Total tokens: {}", format_tokens_with_commas(graph_result.summary.total_tokens)).bright_black());
    println!("{}", format!("    Total cost: {}", format_currency(graph_result.summary.total_cost)).bright_black());
    println!("{}", format!("    Sources: {}", graph_result.summary.sources.join(", ")).bright_black());
    println!("{}", format!("    Models: {} models", graph_result.summary.models.len()).bright_black());
    println!();

    if graph_result.summary.total_tokens == 0 {
        println!("{}", "  No usage data found to submit.\n".yellow());
        return Ok(());
    }

    if dry_run {
        println!("{}", "  Dry run - not submitting data.\n".yellow());
        return Ok(());
    }

    println!("{}", "  Submitting to server...".bright_black());

    let api_url = auth::get_api_base_url();

    let response = rt.block_on(async {
        reqwest::Client::new()
            .post(format!("{}/api/submit", api_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", credentials.token))
            .json(&graph_result)
            .send()
            .await
    });

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body: SubmitResponse = rt.block_on(async { resp.json().await })
                .unwrap_or_else(|_| SubmitResponse {
                    success: false,
                    submission_id: None,
                    username: None,
                    metrics: None,
                    warnings: None,
                    error: Some(format!("Server returned {} with unparseable response", status)),
                    details: None,
                });

            if !status.is_success() || !body.success {
                eprintln!("\n  {}", format!("Error: {}", body.error.unwrap_or_else(|| "Submission failed".to_string())).red());
                if let Some(details) = body.details {
                    for detail in details {
                        eprintln!("{}", format!("    - {}", detail).bright_black());
                    }
                }
                println!();
                std::process::exit(1);
            }

            println!("\n  {}", "Successfully submitted!".green());
            println!();
            println!("{}", "  Summary:".white());
            if let Some(id) = body.submission_id {
                println!("{}", format!("    Submission ID: {}", id).bright_black());
            }
            if let Some(metrics) = &body.metrics {
                if let Some(tokens) = metrics.total_tokens {
                    println!("{}", format!("    Total tokens: {}", format_tokens_with_commas(tokens)).bright_black());
                }
                if let Some(cost) = metrics.total_cost {
                    println!("{}", format!("    Total cost: {}", format_currency(cost)).bright_black());
                }
                if let Some(days) = metrics.active_days {
                    println!("{}", format!("    Active days: {}", days).bright_black());
                }
            }
            println!();
            println!("{}", format!("  View your profile: {}/u/{}", api_url, credentials.username).cyan());
            println!();

            if let Some(warnings) = body.warnings {
                if !warnings.is_empty() {
                    println!("{}", "  Warnings:".yellow());
                    for warning in warnings {
                        println!("{}", format!("    - {}", warning).bright_black());
                    }
                    println!();
                }
            }
        }
        Err(_) => {
            eprintln!("\n  {}", "Error: Failed to connect to server.".red());
            eprintln!("{}", "  Check your internet connection and try again.\n".bright_black());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_cursor_command(subcommand: CursorSubcommand) -> Result<()> {
    match subcommand {
        CursorSubcommand::Login { name } => cursor::run_cursor_login(name),
        CursorSubcommand::Logout {
            name,
            all,
            purge_cache,
        } => cursor::run_cursor_logout(name, all, purge_cache),
        CursorSubcommand::Status { name } => cursor::run_cursor_status(name),
        CursorSubcommand::Accounts { json } => cursor::run_cursor_accounts(json),
        CursorSubcommand::Switch { name } => cursor::run_cursor_switch(&name),
    }
}

fn format_tokens_with_commas(n: i64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

fn run_headless_command(
    source: &str,
    args: Vec<String>,
    format: Option<String>,
    output: Option<String>,
    no_auto_flags: bool,
) -> Result<()> {
    use std::process::Command;
    use std::io::{Write, Read};
    use uuid::Uuid;
    use chrono::Utc;
    
    // Validate source - only "codex" is supported
    let source_lower = source.to_lowercase();
    if source_lower != "codex" {
        eprintln!("\n  Error: Unknown headless source '{}'.", source);
        eprintln!("  Currently only 'codex' is supported.\n");
        std::process::exit(1);
    }
    
    // Determine format (default to jsonl)
    let resolved_format = match format {
        Some(f) if f == "json" || f == "jsonl" => f,
        Some(f) => {
            eprintln!("\n  Error: Invalid format '{}'. Use json or jsonl.\n", f);
            std::process::exit(1);
        }
        None => "jsonl".to_string(),
    };
    
    // Build final args - auto-add --json for codex unless --no-auto-flags
    let mut final_args = args.clone();
    if !no_auto_flags && source_lower == "codex" && !final_args.contains(&"--json".to_string()) {
        final_args.push("--json".to_string());
    }
    
    // Get headless roots
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let headless_roots = get_headless_roots(&home_dir);
    
    // Build output path
    let output_path = if let Some(custom_output) = output {
        // Use custom output path
        let parent = Path::new(&custom_output).parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        custom_output
    } else {
        // Generate timestamped filename in headless directory
        let root = headless_roots.first().cloned().unwrap_or_else(|| {
            home_dir.join(".config/tokscale/headless")
        });
        let dir = root.join(&source_lower);
        std::fs::create_dir_all(&dir)?;
        
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H-%M-%S-%3fZ").to_string();
        let uuid_short = Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect::<String>();
        let filename = format!("{}-{}-{}.{}", source_lower, timestamp, uuid_short, resolved_format);
        
        dir.join(filename).to_string_lossy().to_string()
    };
    
    // Print info
    use colored::Colorize;
    println!("\n  {}", "Headless capture".cyan());
    println!("  {}", format!("source: {}", source_lower).bright_black());
    println!("  {}", format!("output: {}", output_path).bright_black());
    println!();
    
    // Spawn subprocess with inherited stdin/stderr and piped stdout
    let mut child = Command::new(&source_lower)
        .args(&final_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn '{}': {}", source_lower, e))?;
    
    // Get stdout
    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout from command"))?;
    
    // Create output file
    let mut output_file = std::fs::File::create(&output_path)
        .map_err(|e| anyhow::anyhow!("Failed to create output file '{}': {}", output_path, e))?;
    
    // Copy stdout to file
    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = [0; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                output_file.write_all(&buffer[..n])
                    .map_err(|e| anyhow::anyhow!("Failed to write to output file: {}", e))?;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read from subprocess stdout: {}", e));
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for subprocess: {}", e))?;
    
    let exit_code = status.code().unwrap_or(1);
    
    // Print success message
    println!("{}", format!("✓ Saved headless output to {}", output_path).green());
    println!();
    
    // Exit with same code as subprocess
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    
    Ok(())
}
