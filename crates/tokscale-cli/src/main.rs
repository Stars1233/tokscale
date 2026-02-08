mod tui;
mod auth;
mod cursor;

use anyhow::Result;
use chrono::Datelike;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tui::Tab;

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

    #[arg(long, help = "Output as JSON")]
    json: bool,

    #[arg(long, help = "Use legacy CLI table output")]
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

    #[arg(long, help = "Show only Pi usage")]
    pi: bool,

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
        #[arg(long, help = "Show only Pi usage")]
        pi: bool,
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
        #[arg(long, help = "Show only Pi usage")]
        pi: bool,
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
        #[arg(long, help = "Disable spinner")]
        no_spinner: bool,
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
        #[arg(long, help = "Show only Pi usage")]
        pi: bool,
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
        #[arg(long, help = "Show only Pi usage")]
        pi: bool,
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
        #[arg(long, help = "Include only OpenCode data")]
        opencode: bool,
        #[arg(long, help = "Include only Claude Code data")]
        claude: bool,
        #[arg(long, help = "Include only Codex CLI data")]
        codex: bool,
        #[arg(long, help = "Include only Gemini CLI data")]
        gemini: bool,
        #[arg(long, help = "Include only Cursor IDE data")]
        cursor: bool,
        #[arg(long, help = "Include only Amp data")]
        amp: bool,
        #[arg(long, help = "Include only Droid data")]
        droid: bool,
        #[arg(long, help = "Include only OpenClaw data")]
        openclaw: bool,
        #[arg(long, help = "Include only Pi data")]
        pi: bool,
        #[arg(long, help = "Submit only today's usage")]
        today: bool,
        #[arg(long, help = "Submit last 7 days")]
        week: bool,
        #[arg(long, help = "Submit current month")]
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
        #[arg(long, help = "Show only Pi usage")]
        pi: bool,
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
            light,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            pi,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark,
            no_spinner,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            if json || light {
                run_models_report(json, sources, since, until, year, benchmark, no_spinner)
            } else {
                tui::run(&cli.theme, cli.refresh, cli.debug, sources, since, until, year, Some(Tab::Models))
            }
        }
        Some(Commands::Monthly {
            json,
            light,
            opencode,
            claude,
            codex,
            gemini,
            cursor,
            amp,
            droid,
            openclaw,
            pi,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark,
            no_spinner,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            if json || light {
                run_monthly_report(json, sources, since, until, year, benchmark, no_spinner)
            } else {
                tui::run(&cli.theme, cli.refresh, cli.debug, sources, since, until, year, Some(Tab::Daily))
            }
        }
        Some(Commands::Pricing { model_id, json, provider, no_spinner }) => {
            run_pricing_lookup(&model_id, json, provider.as_deref(), no_spinner)
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
            pi,
            today,
            week,
            month,
            since,
            until,
            year,
            benchmark,
            no_spinner,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            run_graph_command(output, sources, since, until, year, benchmark, no_spinner)
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
            pi,
            today,
            week,
            month,
            since,
            until,
            year,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            tui::run(&cli.theme, cli.refresh, cli.debug, sources, since, until, year, None)
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
            pi,
            today,
            week,
            month,
            since,
            until,
            year,
            dry_run,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
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
            pi,
            short,
            agents,
            clients,
            disable_pinned,
            no_spinner: _,
        }) => {
            let sources = build_source_filter(SourceFlags {
                opencode, claude, codex, gemini, cursor, amp, droid, openclaw, pi,
            });
            run_wrapped_command(output, year, sources, short, agents, clients, disable_pinned)
        }
        Some(Commands::Cursor { subcommand }) => {
            run_cursor_command(subcommand)
        }
        None => {
            let sources = build_source_filter(SourceFlags {
                opencode: cli.opencode,
                claude: cli.claude,
                codex: cli.codex,
                gemini: cli.gemini,
                cursor: cli.cursor,
                amp: cli.amp,
                droid: cli.droid,
                openclaw: cli.openclaw,
                pi: cli.pi,
            });
            let (since, until) = build_date_filter(cli.today, cli.week, cli.month, cli.since, cli.until);
            let year = normalize_year_filter(cli.today, cli.week, cli.month, cli.year);

            if cli.json {
                run_models_report(cli.json, sources, since, until, year, cli.benchmark, true)
            } else if cli.light {
                run_models_report(false, sources, since, until, year, cli.benchmark, true)
            } else {
                tui::run(&cli.theme, cli.refresh, cli.debug, sources, since, until, year, None)
            }
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
    pi: bool,
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
    if flags.pi { sources.push("pi".to_string()); }
    
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
    use chrono::{Utc, Datelike, Duration};
    
    // Use UTC for date shortcuts to match TypeScript behavior
    // TS uses: new Date().toISOString().split("T")[0]
    if today {
        let date = Utc::now().format("%Y-%m-%d").to_string();
        return (Some(date.clone()), Some(date));
    }
    
    if week {
        let end = Utc::now();
        let start = end - Duration::days(6);
        return (
            Some(start.format("%Y-%m-%d").to_string()),
            Some(end.format("%Y-%m-%d").to_string()),
        );
    }
    
    if month {
        let now = Utc::now();
        let start = now.with_day(1).unwrap_or(now);
        return (
            Some(start.format("%Y-%m-%d").to_string()),
            Some(now.format("%Y-%m-%d").to_string()),
        );
    }
    
    (since, until)
}

fn normalize_year_filter(
    today: bool,
    week: bool,
    month: bool,
    year: Option<String>,
) -> Option<String> {
    if today || week || month {
        None
    } else {
        year
    }
}

fn run_models_report(
    json: bool,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    benchmark: bool,
    no_spinner: bool,
) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_model_report, ReportOptions};
    use std::time::Instant;

    if !no_spinner {
        eprintln!("  Scanning session data...");
    }
    let start = Instant::now();
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
    let processing_time_ms = start.elapsed().as_millis();

    if json {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ModelUsageJson {
            source: String,
            model: String,
            provider: String,
            input: i64,
            output: i64,
            cache_read: i64,
            cache_write: i64,
            reasoning: i64,
            message_count: i32,
            cost: f64,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ModelReportJson {
            entries: Vec<ModelUsageJson>,
            total_input: i64,
            total_output: i64,
            total_cache_read: i64,
            total_cache_write: i64,
            total_messages: i32,
            total_cost: f64,
            processing_time_ms: u32,
        }

        let output = ModelReportJson {
            entries: report
                .entries
                .into_iter()
                .map(|e| ModelUsageJson {
                    source: e.source,
                    model: e.model,
                    provider: e.provider,
                    input: e.input,
                    output: e.output,
                    cache_read: e.cache_read,
                    cache_write: e.cache_write,
                    reasoning: e.reasoning,
                    message_count: e.message_count,
                    cost: e.cost,
                })
                .collect(),
            total_input: report.total_input,
            total_output: report.total_output,
            total_cache_read: report.total_cache_read,
            total_cache_write: report.total_cache_write,
            total_messages: report.total_messages,
            total_cost: report.total_cost,
            processing_time_ms: report.processing_time_ms,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
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

        if benchmark {
            use colored::Colorize;
            println!("{}", format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black());
        }
    }

    Ok(())
}

fn run_monthly_report(
    json: bool,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    benchmark: bool,
    no_spinner: bool,
) -> Result<()> {
    use tokio::runtime::Runtime;
    use tokscale_core::{get_monthly_report, ReportOptions};
    use std::time::Instant;

    if !no_spinner {
        eprintln!("  Scanning session data...");
    }
    let start = Instant::now();
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
    let processing_time_ms = start.elapsed().as_millis();

    if json {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct MonthlyUsageJson {
            month: String,
            models: Vec<String>,
            input: i64,
            output: i64,
            cache_read: i64,
            cache_write: i64,
            message_count: i32,
            cost: f64,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct MonthlyReportJson {
            entries: Vec<MonthlyUsageJson>,
            total_cost: f64,
            processing_time_ms: u32,
        }

        let output = MonthlyReportJson {
            entries: report
                .entries
                .into_iter()
                .map(|e| MonthlyUsageJson {
                    month: e.month,
                    models: e.models,
                    input: e.input,
                    output: e.output,
                    cache_read: e.cache_read,
                    cache_write: e.cache_write,
                    message_count: e.message_count,
                    cost: e.cost,
                })
                .collect(),
            total_cost: report.total_cost,
            processing_time_ms: report.processing_time_ms,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
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

        if benchmark {
            use colored::Colorize;
            println!("{}", format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black());
        }
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
    
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::io::Read;
    
    let settings = tui::settings::Settings::load();
    let timeout = settings.get_native_timeout();
    
    println!("{}", format!("  timeout: {}s", timeout.as_secs()).bright_black());
    println!();
    
    let mut child = Command::new(runtime)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn wrapped generator: {}", e))?;
    
    let timed_out = Arc::new(AtomicBool::new(false));
    let timed_out_clone = Arc::clone(&timed_out);
    let child_id = child.id();
    
    // Watchdog: kills subprocess after timeout expires
    let timeout_handle = std::thread::spawn(move || {
        std::thread::sleep(timeout);
        if !timed_out_clone.load(Ordering::SeqCst) {
            timed_out_clone.store(true, Ordering::SeqCst);
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(child_id.to_string())
                    .output();
            }
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/PID", &child_id.to_string()])
                    .output();
            }
        }
    });
    
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    
    let mut stdout_content = String::new();
    let mut stderr_content = String::new();
    
    if let Some(mut out) = stdout {
        let _ = out.read_to_string(&mut stdout_content);
    }
    if let Some(mut err) = stderr {
        let _ = err.read_to_string(&mut stderr_content);
    }
    
    let status = child.wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for wrapped generator: {}", e))?;
    
    timed_out.store(true, Ordering::SeqCst);
    let _ = timeout_handle.join();
    
    if timed_out.load(Ordering::SeqCst) && !status.success() && status.code().is_none() {
        eprintln!("{}", format!("\n  Wrapped generator timed out after {}s", timeout.as_secs()).red());
        eprintln!("{}", "  Increase timeout with TOKSCALE_NATIVE_TIMEOUT_MS or settings.json".bright_black());
        println!();
        std::process::exit(124);
    }
    
    if !status.success() {
        eprintln!("{}", "\nError generating wrapped image:".red());
        eprintln!("{}", stderr_content);
        std::process::exit(status.code().unwrap_or(1));
    }
    
    // Parse output to get the file path
    let output_path = stdout_content.trim().to_string();
    
    if !output_path.is_empty() {
        println!("{}", format!("\n  ✓ Generated wrapped image: {}\n", output_path).green());
    } else {
        println!("{}", "\n  ✓ Wrapped image generated successfully\n".green());
    }
    
    Ok(())
}

fn run_pricing_lookup(model_id: &str, json: bool, provider: Option<&str>, no_spinner: bool) -> Result<()> {
    use indicatif::ProgressBar;
    use indicatif::ProgressStyle;
    use tokio::runtime::Runtime;
    use tokscale_core::pricing::PricingService;
    use colored::Colorize;

    let provider_normalized = provider.map(|p| p.to_lowercase());
    if let Some(ref p) = provider_normalized {
        if p != "litellm" && p != "openrouter" {
            println!("\n  {}", format!("Invalid provider: {}", provider.unwrap_or("")).red());
            println!("{}\n", "  Valid providers: litellm, openrouter".bright_black());
            std::process::exit(1);
        }
    }

    let spinner = if no_spinner {
        None
    } else {
        let provider_label = provider
            .map(|p| format!(" from {}", p))
            .unwrap_or_default();
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner());
        pb.set_message(format!("Fetching pricing data{}...", provider_label));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    };

    let rt = Runtime::new()?;
    let result = match rt.block_on(async {
        let svc = PricingService::get_or_init().await?;
        Ok::<_, String>(svc.lookup_with_source(model_id, provider_normalized.as_deref()))
    }) {
        Ok(result) => result,
        Err(err) => {
            if let Some(pb) = spinner {
                pb.finish_and_clear();
            }
            if json {
                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct ErrorOutput {
                    error: String,
                    model_id: String,
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ErrorOutput {
                        error: err,
                        model_id: model_id.to_string(),
                    })?
                );
                std::process::exit(1);
            }
            return Err(anyhow::anyhow!(err));
        }
    };

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if json {
        match result {
            Some(pricing) => {
                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct PricingValues {
                    input_cost_per_token: f64,
                    output_cost_per_token: f64,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    cache_read_input_token_cost: Option<f64>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    cache_creation_input_token_cost: Option<f64>,
                }

                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct PricingOutput {
                    model_id: String,
                    matched_key: String,
                    source: String,
                    pricing: PricingValues,
                }

                let output = PricingOutput {
                    model_id: model_id.to_string(),
                    matched_key: pricing.matched_key,
                    source: pricing.source,
                    pricing: PricingValues {
                        input_cost_per_token: pricing.pricing.input_cost_per_token.unwrap_or(0.0),
                        output_cost_per_token: pricing.pricing.output_cost_per_token.unwrap_or(0.0),
                        cache_read_input_token_cost: pricing.pricing.cache_read_input_token_cost,
                        cache_creation_input_token_cost: pricing.pricing.cache_creation_input_token_cost,
                    },
                };

                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            None => {
                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct ErrorOutput {
                    error: String,
                    model_id: String,
                }

                let output = ErrorOutput {
                    error: "Model not found".to_string(),
                    model_id: model_id.to_string(),
                };

                println!("{}", serde_json::to_string_pretty(&output)?);
                std::process::exit(1);
            }
        }
    } else {
        match result {
            Some(pricing) => {
                println!("\n  Pricing for: {}", model_id.bold());
                println!("  Matched key: {}", pricing.matched_key);
                let source_label = if pricing.source.eq_ignore_ascii_case("litellm") {
                    "LiteLLM"
                } else {
                    "OpenRouter"
                };
                println!("  Source: {}", source_label);
                println!();
                let input = pricing.pricing.input_cost_per_token.unwrap_or(0.0);
                let output = pricing.pricing.output_cost_per_token.unwrap_or(0.0);
                println!("  Input:  ${:.2} / 1M tokens", input * 1_000_000.0);
                println!("  Output: ${:.2} / 1M tokens", output * 1_000_000.0);
                if let Some(cache_read) = pricing.pricing.cache_read_input_token_cost {
                    println!("  Cache Read:  ${:.2} / 1M tokens", cache_read * 1_000_000.0);
                }
                if let Some(cache_write) = pricing.pricing.cache_creation_input_token_cost {
                    println!("  Cache Write: ${:.2} / 1M tokens", cache_write * 1_000_000.0);
                }
                println!();
            }
            None => {
                println!("\n  {}\n", format!("Model not found: {}", model_id).red());
                std::process::exit(1);
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
            "pi".to_string(),
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
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
    struct LegacyPath {
        path: String,
        exists: bool,
    }
    
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
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
        SourceRow {
            source: "pi".to_string(),
            label: "Pi".to_string(),
            sessions_path: home_dir.join(".pi/agent/sessions").to_string_lossy().to_string(),
            sessions_path_exists: home_dir.join(".pi/agent/sessions").exists(),
            legacy_paths: vec![],
            message_count: parsed.pi_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
    ];
    
    if json {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
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
    let path_display = if let Some(home) = dirs::home_dir() {
        path.replace(&home.to_string_lossy().to_string(), "~")
    } else {
        path.to_string()
    };
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsTokenBreakdown {
    input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
    reasoning: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsSourceContribution {
    source: String,
    model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_id: Option<String>,
    tokens: TsTokenBreakdown,
    cost: f64,
    messages: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsDailyTotals {
    tokens: i64,
    cost: f64,
    messages: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsDailyContribution {
    date: String,
    totals: TsDailyTotals,
    intensity: u8,
    token_breakdown: TsTokenBreakdown,
    sources: Vec<TsSourceContribution>,
}

#[derive(serde::Serialize)]
struct DateRange {
    start: String,
    end: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsYearSummary {
    year: String,
    total_tokens: i64,
    total_cost: f64,
    range: DateRange,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsDataSummary {
    total_tokens: i64,
    total_cost: f64,
    total_days: i32,
    active_days: i32,
    average_per_day: f64,
    max_cost_in_single_day: f64,
    sources: Vec<String>,
    models: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsExportMeta {
    generated_at: String,
    version: String,
    date_range: DateRange,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TsTokenContributionData {
    meta: TsExportMeta,
    summary: TsDataSummary,
    years: Vec<TsYearSummary>,
    contributions: Vec<TsDailyContribution>,
}

fn to_ts_token_contribution_data(graph: &tokscale_core::GraphResult) -> TsTokenContributionData {
    TsTokenContributionData {
        meta: TsExportMeta {
            generated_at: graph.meta.generated_at.clone(),
            version: graph.meta.version.clone(),
            date_range: DateRange {
                start: graph.meta.date_range_start.clone(),
                end: graph.meta.date_range_end.clone(),
            },
        },
        summary: TsDataSummary {
            total_tokens: graph.summary.total_tokens,
            total_cost: graph.summary.total_cost,
            total_days: graph.summary.total_days,
            active_days: graph.summary.active_days,
            average_per_day: graph.summary.average_per_day,
            max_cost_in_single_day: graph.summary.max_cost_in_single_day,
            sources: graph.summary.sources.clone(),
            models: graph.summary.models.clone(),
        },
        years: graph
            .years
            .iter()
            .map(|y| TsYearSummary {
                year: y.year.clone(),
                total_tokens: y.total_tokens,
                total_cost: y.total_cost,
                range: DateRange {
                    start: y.range_start.clone(),
                    end: y.range_end.clone(),
                },
            })
            .collect(),
        contributions: graph
            .contributions
            .iter()
            .map(|d| TsDailyContribution {
                date: d.date.clone(),
                totals: TsDailyTotals {
                    tokens: d.totals.tokens,
                    cost: d.totals.cost,
                    messages: d.totals.messages,
                },
                intensity: d.intensity,
                token_breakdown: TsTokenBreakdown {
                    input: d.token_breakdown.input,
                    output: d.token_breakdown.output,
                    cache_read: d.token_breakdown.cache_read,
                    cache_write: d.token_breakdown.cache_write,
                    reasoning: d.token_breakdown.reasoning,
                },
                sources: d
                    .sources
                    .iter()
                    .map(|s| TsSourceContribution {
                        source: s.source.clone(),
                        model_id: s.model_id.clone(),
                        provider_id: if s.provider_id.is_empty() {
                            None
                        } else {
                            Some(s.provider_id.clone())
                        },
                        tokens: TsTokenBreakdown {
                            input: s.tokens.input,
                            output: s.tokens.output,
                            cache_read: s.tokens.cache_read,
                            cache_write: s.tokens.cache_write,
                            reasoning: s.tokens.reasoning,
                        },
                        cost: s.cost,
                        messages: s.messages,
                    })
                    .collect(),
            })
            .collect(),
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

fn prompt_star_repo() -> Result<()> {
    use colored::Colorize;
    use std::io::{self, Write};
    use std::process::Command;

    let gh_available = Command::new("gh")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if !gh_available {
        return Ok(());
    }

    println!("{}", "  Please consider starring tokscale on GitHub!".bright_black());
    print!("  Star now with gh CLI? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        println!();
        return Ok(());
    }

    let status = Command::new("gh")
        .args(["repo", "star", "junhoyeo/tokscale"])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("{}", "  ✓ Starred! Thank you for your support.".green());
            println!();
        }
        _ => {
            println!("{}", "  Failed to star via gh CLI. Continuing to submit...".yellow());
            println!();
        }
    }

    Ok(())
}

fn run_graph_command(
    output: Option<String>,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    benchmark: bool,
    no_spinner: bool,
) -> Result<()> {
    use colored::Colorize;
    use tokscale_core::{generate_graph, ReportOptions};
    use std::time::Instant;

    let show_progress = output.is_some() && !no_spinner;
    let include_cursor = sources.as_ref().map_or(true, |s| s.iter().any(|src| src == "cursor"));
    let has_cursor_cache = cursor::has_cursor_usage_cache();
    let mut cursor_sync_result: Option<cursor::SyncCursorResult> = None;

    if include_cursor && cursor::is_cursor_logged_in() {
        let rt_sync = tokio::runtime::Runtime::new()?;
        cursor_sync_result = Some(rt_sync.block_on(async { cursor::sync_cursor_cache().await }));
    }

    if show_progress {
        eprintln!("  Scanning session data...");
    }
    let start = Instant::now();

    if show_progress {
        eprintln!("  Generating graph data...");
    }
    let rt = tokio::runtime::Runtime::new()?;
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

    let processing_time_ms = start.elapsed().as_millis() as u32;
    let output_data = to_ts_token_contribution_data(&graph_result);
    let json_output = serde_json::to_string_pretty(&output_data)?;
    
    if let Some(output_path) = output {
        std::fs::write(&output_path, json_output)?;

        eprintln!("{}", format!("✓ Graph data written to {}", output_path).green());
        eprintln!("{}", format!("  {} days, {} sources, {} models",
            output_data.contributions.len(),
            output_data.summary.sources.len(),
            output_data.summary.models.len()
        ).bright_black());
        eprintln!("{}", format!("  Total: {}", format_currency(output_data.summary.total_cost)).bright_black());
        
        if benchmark {
            eprintln!("{}", format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black());
            if let Some(sync) = cursor_sync_result {
                if sync.synced {
                    eprintln!("{}", format!("  Cursor: {} usage events synced (full lifetime data)", sync.rows).bright_black());
                } else if let Some(err) = sync.error {
                    if has_cursor_cache {
                        eprintln!("{}", format!("  Cursor: sync failed - {}", err).yellow());
                    }
                }
            }
        }
    } else {
        println!("{}", json_output);
    }
    
    Ok(())
}

#[derive(serde::Deserialize)]
struct SubmitResponse {
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
    use std::io::IsTerminal;
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

    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        let _ = prompt_star_repo();
    }

    println!("\n  {}\n", "Tokscale - Submit Usage Data".cyan());
    
    let include_cursor = sources.as_ref().map_or(true, |s| s.iter().any(|src| src == "cursor"));
    let has_cursor_cache = cursor::has_cursor_usage_cache();
    if include_cursor && cursor::is_cursor_logged_in() {
        println!("{}", "  Syncing Cursor usage data...".bright_black());
        let rt_sync = Runtime::new()?;
        let sync_result = rt_sync.block_on(async { cursor::sync_cursor_cache().await });
        if sync_result.synced {
            println!("{}", format!("  Cursor: {} usage events synced", sync_result.rows).bright_black());
        } else if let Some(err) = sync_result.error {
            if has_cursor_cache {
                println!("{}", format!("  Cursor sync failed; using cached data: {}", err).yellow());
            }
        }
    }
    
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

    let submit_payload = to_ts_token_contribution_data(&graph_result);

    let response = rt.block_on(async {
        reqwest::Client::new()
            .post(format!("{}/api/submit", api_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", credentials.token))
            .json(&submit_payload)
            .send()
            .await
    });

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body: SubmitResponse = rt.block_on(async { resp.json().await })
                .unwrap_or_else(|_| SubmitResponse {
                    submission_id: None,
                    username: None,
                    metrics: None,
                    warnings: None,
                    error: Some(format!("Server returned {} with unparseable response", status)),
                    details: None,
                });

            if !status.is_success() {
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
        Err(err) => {
            eprintln!("\n  {}", "Error: Failed to connect to server.".red());
            eprintln!("{}\n", format!("  {}", err).bright_black());
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;
    
    let source_lower = source.to_lowercase();
    if source_lower != "codex" {
        eprintln!("\n  Error: Unknown headless source '{}'.", source);
        eprintln!("  Currently only 'codex' is supported.\n");
        std::process::exit(1);
    }
    
    let resolved_format = match format {
        Some(f) if f == "json" || f == "jsonl" => f,
        Some(f) => {
            eprintln!("\n  Error: Invalid format '{}'. Use json or jsonl.\n", f);
            std::process::exit(1);
        }
        None => "jsonl".to_string(),
    };
    
    let mut final_args = args.clone();
    if !no_auto_flags && source_lower == "codex" && !final_args.contains(&"--json".to_string()) {
        final_args.push("--json".to_string());
    }
    
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let headless_roots = get_headless_roots(&home_dir);
    
    let output_path = if let Some(custom_output) = output {
        let parent = Path::new(&custom_output).parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        custom_output
    } else {
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
    
    let settings = tui::settings::Settings::load();
    let timeout = settings.get_native_timeout();
    
    use colored::Colorize;
    println!("\n  {}", "Headless capture".cyan());
    println!("  {}", format!("source: {}", source_lower).bright_black());
    println!("  {}", format!("output: {}", output_path).bright_black());
    println!("  {}", format!("timeout: {}s", timeout.as_secs()).bright_black());
    println!();
    
    let mut child = Command::new(&source_lower)
        .args(&final_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn '{}': {}", source_lower, e))?;
    
    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout from command"))?;
    
    let mut output_file = std::fs::File::create(&output_path)
        .map_err(|e| anyhow::anyhow!("Failed to create output file '{}': {}", output_path, e))?;
    
    let timed_out = Arc::new(AtomicBool::new(false));
    let timed_out_clone = Arc::clone(&timed_out);
    let child_id = child.id();
    
    let timeout_handle = std::thread::spawn(move || {
        std::thread::sleep(timeout);
        if !timed_out_clone.load(Ordering::SeqCst) {
            timed_out_clone.store(true, Ordering::SeqCst);
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let _ = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(child_id.to_string())
                    .exec();
            }
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/PID", &child_id.to_string()])
                    .output();
            }
        }
    });
    
    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = [0; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                output_file.write_all(&buffer[..n])
                    .map_err(|e| anyhow::anyhow!("Failed to write to output file: {}", e))?;
            }
            Err(e) => {
                if timed_out.load(Ordering::SeqCst) {
                    break;
                }
                return Err(anyhow::anyhow!("Failed to read from subprocess stdout: {}", e));
            }
        }
    }
    
    let status = child.wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for subprocess: {}", e))?;
    
    timed_out.store(true, Ordering::SeqCst);
    let _ = timeout_handle.join();
    
    if timed_out.load(Ordering::SeqCst) && !status.success() {
        eprintln!("{}", format!("\n  Subprocess timed out after {}s", timeout.as_secs()).red());
        eprintln!("{}", "  Partial output saved. Increase timeout with TOKSCALE_NATIVE_TIMEOUT_MS or settings.json".bright_black());
        println!();
        std::process::exit(124);
    }
    
    let exit_code = status.code().unwrap_or(1);
    
    println!("{}", format!("✓ Saved headless output to {}", output_path).green());
    println!();
    
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    
    Ok(())
}
