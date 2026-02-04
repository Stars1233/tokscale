mod tui;
mod auth;

use anyhow::Result;
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
    Pricing { model_id: String },
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
        Some(Commands::Pricing { model_id }) => {
            run_pricing_lookup(&model_id)
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
        None => {
            tui::run(&cli.theme, cli.refresh, cli.debug)
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
