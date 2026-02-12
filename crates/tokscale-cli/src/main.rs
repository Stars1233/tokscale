mod auth;
mod commands;
mod cursor;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
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

    #[arg(long, help = "Disable spinner (for AI agents and scripts)")]
    no_spinner: bool,
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
        #[arg(
            long,
            help = "Show what would be submitted without actually submitting"
        )]
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
        #[arg(
            long,
            help = "Display total tokens in abbreviated format (e.g., 7.14B)"
        )]
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
    use std::io::IsTerminal;

    let cli = Cli::parse();
    let can_use_tui = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            if json || light || !can_use_tui {
                run_models_report(
                    json,
                    sources,
                    since,
                    until,
                    year,
                    benchmark,
                    no_spinner || !can_use_tui,
                    today,
                    week,
                    month,
                )
            } else {
                tui::run(
                    &cli.theme,
                    cli.refresh,
                    cli.debug,
                    sources,
                    since,
                    until,
                    year,
                    Some(Tab::Models),
                )
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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            if json || light || !can_use_tui {
                run_monthly_report(
                    json,
                    sources,
                    since,
                    until,
                    year,
                    benchmark,
                    no_spinner || !can_use_tui,
                    today,
                    week,
                    month,
                )
            } else {
                tui::run(
                    &cli.theme,
                    cli.refresh,
                    cli.debug,
                    sources,
                    since,
                    until,
                    year,
                    Some(Tab::Daily),
                )
            }
        }
        Some(Commands::Pricing {
            model_id,
            json,
            provider,
            no_spinner,
        }) => run_pricing_lookup(&model_id, json, provider.as_deref(), no_spinner),
        Some(Commands::Sources { json }) => run_sources_command(json),
        Some(Commands::Login) => run_login_command(),
        Some(Commands::Logout) => run_logout_command(),
        Some(Commands::Whoami) => run_whoami_command(),
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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
            });
            let (since, until) = build_date_filter(today, week, month, since, until);
            let year = normalize_year_filter(today, week, month, year);
            tui::run(
                &cli.theme,
                cli.refresh,
                cli.debug,
                sources,
                since,
                until,
                year,
                None,
            )
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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
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
        }) => run_headless_command(&source, args, format, output, no_auto_flags),
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
                opencode,
                claude,
                codex,
                gemini,
                cursor,
                amp,
                droid,
                openclaw,
                pi,
            });
            run_wrapped_command(
                output,
                year,
                sources,
                short,
                agents,
                clients,
                disable_pinned,
            )
        }
        Some(Commands::Cursor { subcommand }) => run_cursor_command(subcommand),
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
            let (since, until) =
                build_date_filter(cli.today, cli.week, cli.month, cli.since, cli.until);
            let year = normalize_year_filter(cli.today, cli.week, cli.month, cli.year);

            if cli.json {
                run_models_report(
                    cli.json,
                    sources,
                    since,
                    until,
                    year,
                    cli.benchmark,
                    cli.no_spinner || cli.json,
                    cli.today,
                    cli.week,
                    cli.month,
                )
            } else if cli.light || !can_use_tui {
                run_models_report(
                    false,
                    sources,
                    since,
                    until,
                    year,
                    cli.benchmark,
                    cli.no_spinner || !can_use_tui,
                    cli.today,
                    cli.week,
                    cli.month,
                )
            } else {
                tui::run(
                    &cli.theme,
                    cli.refresh,
                    cli.debug,
                    sources,
                    since,
                    until,
                    year,
                    None,
                )
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
    if flags.opencode {
        sources.push("opencode".to_string());
    }
    if flags.claude {
        sources.push("claude".to_string());
    }
    if flags.codex {
        sources.push("codex".to_string());
    }
    if flags.gemini {
        sources.push("gemini".to_string());
    }
    if flags.cursor {
        sources.push("cursor".to_string());
    }
    if flags.amp {
        sources.push("amp".to_string());
    }
    if flags.droid {
        sources.push("droid".to_string());
    }
    if flags.openclaw {
        sources.push("openclaw".to_string());
    }
    if flags.pi {
        sources.push("pi".to_string());
    }

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
    use chrono::{Datelike, Duration, Utc};

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

fn get_date_range_label(
    today: bool,
    week: bool,
    month: bool,
    since: &Option<String>,
    until: &Option<String>,
    year: &Option<String>,
) -> Option<String> {
    if today {
        return Some("Today".to_string());
    }
    if week {
        return Some("Last 7 days".to_string());
    }
    if month {
        let now = chrono::Utc::now();
        return Some(now.format("%B %Y").to_string());
    }
    if let Some(y) = year {
        return Some(y.clone());
    }
    let mut parts = Vec::new();
    if let Some(s) = since {
        parts.push(format!("from {}", s));
    }
    if let Some(u) = until {
        parts.push(format!("to {}", u));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

struct LightSpinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

const TABLE_PRESET: &str = "││──├─┼┤│─┼├┤┬┴┌┐└┘";

impl LightSpinner {
    const WIDTH: usize = 8;
    const HOLD_START: usize = 30;
    const HOLD_END: usize = 9;
    const TRAIL_LENGTH: usize = 4;
    const TRAIL_COLORS: [u8; 6] = [51, 44, 37, 30, 23, 17];
    const INACTIVE_COLOR: u8 = 240;
    const FRAME_MS: u64 = 40;

    fn start(message: &'static str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_thread = Arc::clone(&running);
        let message = message.to_string();

        let handle = thread::spawn(move || {
            let mut frame = 0usize;
            let mut stderr = io::stderr().lock();

            let _ = write!(stderr, "\x1b[?25l");
            let _ = stderr.flush();

            while running_thread.load(Ordering::Relaxed) {
                let spinner = Self::frame(frame);
                let _ = write!(stderr, "\r\x1b[K  {} {}", spinner, message);
                let _ = stderr.flush();
                frame = frame.wrapping_add(1);
                thread::sleep(Duration::from_millis(Self::FRAME_MS));
            }

            let _ = write!(stderr, "\r\x1b[K\x1b[?25h");
            let _ = stderr.flush();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    fn stop(mut self) {
        self.stop_inner();
    }

    fn stop_inner(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn frame(frame: usize) -> String {
        let (position, forward) = Self::scanner_state(frame);
        let mut out = String::new();

        for i in 0..Self::WIDTH {
            let distance = if forward {
                if position >= i {
                    position - i
                } else {
                    usize::MAX
                }
            } else if i >= position {
                i - position
            } else {
                usize::MAX
            };

            if distance < Self::TRAIL_LENGTH {
                let color = Self::TRAIL_COLORS[distance.min(Self::TRAIL_COLORS.len() - 1)];
                out.push_str(&format!("\x1b[38;5;{}m■\x1b[0m", color));
            } else {
                out.push_str(&format!("\x1b[38;5;{}m⬝\x1b[0m", Self::INACTIVE_COLOR));
            }
        }

        out
    }

    fn scanner_state(frame: usize) -> (usize, bool) {
        let forward_frames = Self::WIDTH;
        let backward_frames = Self::WIDTH - 1;
        let total_cycle = forward_frames + Self::HOLD_END + backward_frames + Self::HOLD_START;
        let normalized = frame % total_cycle;

        if normalized < forward_frames {
            (normalized, true)
        } else if normalized < forward_frames + Self::HOLD_END {
            (Self::WIDTH - 1, true)
        } else if normalized < forward_frames + Self::HOLD_END + backward_frames {
            (
                Self::WIDTH - 2 - (normalized - forward_frames - Self::HOLD_END),
                false,
            )
        } else {
            (0, false)
        }
    }
}

impl Drop for LightSpinner {
    fn drop(&mut self) {
        self.stop_inner();
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
    today: bool,
    week: bool,
    month_flag: bool,
) -> Result<()> {
    use std::time::Instant;
    use tokio::runtime::Runtime;
    use tokscale_core::{get_model_report, ReportOptions};

    let date_range = get_date_range_label(today, week, month_flag, &since, &until, &year);

    let spinner = if no_spinner {
        None
    } else {
        Some(LightSpinner::start("Scanning session data..."))
    };
    let start = Instant::now();
    let rt = Runtime::new()?;
    let report = rt
        .block_on(async {
            get_model_report(ReportOptions {
                home_dir: None,
                sources,
                since,
                until,
                year,
            })
            .await
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(spinner) = spinner {
        spinner.stop();
    }

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
        use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

        let term_width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(120);
        let compact = term_width < 100;

        let mut table = Table::new();
        table.load_preset(TABLE_PRESET);
        table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
        if compact {
            table.set_header(vec![
                Cell::new("Source/Model").fg(Color::Cyan),
                Cell::new("Models").fg(Color::Cyan),
                Cell::new("Input").fg(Color::Cyan),
                Cell::new("Output").fg(Color::Cyan),
                Cell::new("Cost").fg(Color::Cyan),
            ]);

            for entry in &report.entries {
                let short_model = format_model_name(&entry.model);
                let source_model = format!(
                    "\x1b[2m{}\x1b[0m {}",
                    capitalize_source(&entry.source),
                    short_model
                );
                let models_col = format!("- {}", short_model);

                table.add_row(vec![
                    Cell::new(source_model),
                    Cell::new(models_col),
                    Cell::new(format_tokens_with_commas(entry.input)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.output)).set_alignment(CellAlignment::Right),
                    Cell::new(format_currency(entry.cost)).set_alignment(CellAlignment::Right),
                ]);
            }

            table.add_row(vec![
                Cell::new("Total")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new(""),
                Cell::new(format_tokens_with_commas(report.total_input))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(report.total_output))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_currency(report.total_cost))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
            ]);
        } else {
            table.set_header(vec![
                Cell::new("Source/Model").fg(Color::Cyan),
                Cell::new("Models").fg(Color::Cyan),
                Cell::new("Input").fg(Color::Cyan),
                Cell::new("Output").fg(Color::Cyan),
                Cell::new("Cache Write").fg(Color::Cyan),
                Cell::new("Cache Read").fg(Color::Cyan),
                Cell::new("Total").fg(Color::Cyan),
                Cell::new("Cost").fg(Color::Cyan),
            ]);

            for entry in &report.entries {
                let short_model = format_model_name(&entry.model);
                let source_model = format!(
                    "\x1b[2m{}\x1b[0m {}",
                    capitalize_source(&entry.source),
                    short_model
                );
                let models_col = format!("- {}", short_model);
                let total = entry.input + entry.output + entry.cache_write + entry.cache_read;

                table.add_row(vec![
                    Cell::new(source_model),
                    Cell::new(models_col),
                    Cell::new(format_tokens_with_commas(entry.input)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.output)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.cache_write))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.cache_read))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(total)).set_alignment(CellAlignment::Right),
                    Cell::new(format_currency(entry.cost)).set_alignment(CellAlignment::Right),
                ]);
            }

            let total_all = report.total_input
                + report.total_output
                + report.total_cache_write
                + report.total_cache_read;
            table.add_row(vec![
                Cell::new("Total")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new(""),
                Cell::new(format_tokens_with_commas(report.total_input))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(report.total_output))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(report.total_cache_write))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(report.total_cache_read))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(total_all))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_currency(report.total_cost))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
            ]);
        }

        let title = match &date_range {
            Some(range) => format!("Token Usage Report by Model ({})", range),
            None => "Token Usage Report by Model".to_string(),
        };
        println!("\n  \x1b[36m{}\x1b[0m\n", title);
        println!("{}", dim_borders(&table.to_string()));

        let total_tokens =
            report.total_input + report.total_output + report.total_cache_write + report.total_cache_read;
        println!(
            "\x1b[90m\n  Total: {} messages, {} tokens, \x1b[32m{}\x1b[90m\x1b[0m",
            format_tokens_with_commas(report.total_messages as i64),
            format_tokens_with_commas(total_tokens),
            format_currency(report.total_cost)
        );

        if benchmark {
            use colored::Colorize;
            println!(
                "{}",
                format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black()
            );
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
    today: bool,
    week: bool,
    month_flag: bool,
) -> Result<()> {
    use std::time::Instant;
    use tokio::runtime::Runtime;
    use tokscale_core::{get_monthly_report, ReportOptions};

    let date_range = get_date_range_label(today, week, month_flag, &since, &until, &year);

    let spinner = if no_spinner {
        None
    } else {
        Some(LightSpinner::start("Scanning session data..."))
    };
    let start = Instant::now();
    let rt = Runtime::new()?;
    let report = rt
        .block_on(async {
            get_monthly_report(ReportOptions {
                home_dir: None,
                sources,
                since,
                until,
                year,
            })
            .await
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(spinner) = spinner {
        spinner.stop();
    }

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
        use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

        let term_width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(120);
        let compact = term_width < 100;

        let mut table = Table::new();
        table.load_preset(TABLE_PRESET);
        table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
        if compact {
            table.set_header(vec![
                Cell::new("Month").fg(Color::Cyan),
                Cell::new("Models").fg(Color::Cyan),
                Cell::new("Input").fg(Color::Cyan),
                Cell::new("Output").fg(Color::Cyan),
                Cell::new("Cost").fg(Color::Cyan),
            ]);

            for entry in &report.entries {
                let models_col = if entry.models.is_empty() {
                    "-".to_string()
                } else {
                    let mut unique_models: Vec<String> = entry
                        .models
                        .iter()
                        .map(|model| format_model_name(model))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    unique_models.sort();
                    unique_models
                        .iter()
                        .map(|m| format!("- {}", m))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                table.add_row(vec![
                    Cell::new(entry.month.clone()),
                    Cell::new(models_col),
                    Cell::new(format_tokens_with_commas(entry.input)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.output)).set_alignment(CellAlignment::Right),
                    Cell::new(format_currency(entry.cost)).set_alignment(CellAlignment::Right),
                ]);
            }

            table.add_row(vec![
                Cell::new("Total")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new(""),
                Cell::new(format_tokens_with_commas(report.entries.iter().map(|e| e.input).sum()))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(report.entries.iter().map(|e| e.output).sum()))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_currency(report.total_cost))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
            ]);
        } else {
            table.set_header(vec![
                Cell::new("Month").fg(Color::Cyan),
                Cell::new("Models").fg(Color::Cyan),
                Cell::new("Input").fg(Color::Cyan),
                Cell::new("Output").fg(Color::Cyan),
                Cell::new("Cache Write").fg(Color::Cyan),
                Cell::new("Cache Read").fg(Color::Cyan),
                Cell::new("Total").fg(Color::Cyan),
                Cell::new("Cost").fg(Color::Cyan),
            ]);

            for entry in &report.entries {
                let models_col = if entry.models.is_empty() {
                    "-".to_string()
                } else {
                    let mut unique_models: Vec<String> = entry
                        .models
                        .iter()
                        .map(|model| format_model_name(model))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    unique_models.sort();
                    unique_models
                        .iter()
                        .map(|m| format!("- {}", m))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                let total = entry.input + entry.output + entry.cache_write + entry.cache_read;

                table.add_row(vec![
                    Cell::new(entry.month.clone()),
                    Cell::new(models_col),
                    Cell::new(format_tokens_with_commas(entry.input)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.output)).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.cache_write))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(entry.cache_read))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens_with_commas(total)).set_alignment(CellAlignment::Right),
                    Cell::new(format_currency(entry.cost)).set_alignment(CellAlignment::Right),
                ]);
            }

            let total_input: i64 = report.entries.iter().map(|e| e.input).sum();
            let total_output: i64 = report.entries.iter().map(|e| e.output).sum();
            let total_cache_write: i64 = report.entries.iter().map(|e| e.cache_write).sum();
            let total_cache_read: i64 = report.entries.iter().map(|e| e.cache_read).sum();
            let total_all = total_input + total_output + total_cache_write + total_cache_read;

            table.add_row(vec![
                Cell::new("Total")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new(""),
                Cell::new(format_tokens_with_commas(total_input))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(total_output))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(total_cache_write))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(total_cache_read))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_tokens_with_commas(total_all))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_currency(report.total_cost))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
            ]);
        }

        let title = match &date_range {
            Some(range) => format!("Monthly Token Usage Report ({})", range),
            None => "Monthly Token Usage Report".to_string(),
        };
        println!("\n  \x1b[36m{}\x1b[0m\n", title);
        println!("{}", dim_borders(&table.to_string()));

        println!(
            "\x1b[90m\n  Total Cost: \x1b[32m{}\x1b[90m\x1b[0m",
            format_currency(report.total_cost)
        );

        if benchmark {
            use colored::Colorize;
            println!(
                "{}",
                format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black()
            );
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

    println!("{}", "\n  Tokscale - Generate Wrapped Image\n".cyan());

    println!("{}", "  Generating wrapped image...".bright_black());
    println!();

    let include_agents = !clients || agents;
    let wrapped_options = commands::wrapped::WrappedOptions {
        output,
        year,
        sources,
        short,
        include_agents,
        pin_sisyphus: !disable_pinned,
    };

    match commands::wrapped::run(wrapped_options) {
        Ok(output_path) => {
            println!(
                "{}",
                format!("\n  ✓ Generated wrapped image: {}\n", output_path).green()
            );
        }
        Err(err) => {
            eprintln!("{}", "\nError generating wrapped image:".red());
            eprintln!("{}", format!("  {}\n", err));
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_pricing_lookup(
    model_id: &str,
    json: bool,
    provider: Option<&str>,
    no_spinner: bool,
) -> Result<()> {
    use colored::Colorize;
    use indicatif::ProgressBar;
    use indicatif::ProgressStyle;
    use tokio::runtime::Runtime;
    use tokscale_core::pricing::PricingService;

    let provider_normalized = provider.map(|p| p.to_lowercase());
    if let Some(ref p) = provider_normalized {
        if p != "litellm" && p != "openrouter" {
            println!(
                "\n  {}",
                format!("Invalid provider: {}", provider.unwrap_or("")).red()
            );
            println!(
                "{}\n",
                "  Valid providers: litellm, openrouter".bright_black()
            );
            std::process::exit(1);
        }
    }

    let spinner = if no_spinner {
        None
    } else {
        let provider_label = provider.map(|p| format!(" from {}", p)).unwrap_or_default();
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
                        cache_creation_input_token_cost: pricing
                            .pricing
                            .cache_creation_input_token_cost,
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
                    println!(
                        "  Cache Read:  ${:.2} / 1M tokens",
                        cache_read * 1_000_000.0
                    );
                }
                if let Some(cache_write) = pricing.pricing.cache_creation_input_token_cost {
                    println!(
                        "  Cache Write: ${:.2} / 1M tokens",
                        cache_write * 1_000_000.0
                    );
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

fn format_currency(n: f64) -> String {
    format!("${:.2}", n)
}

fn dim_borders(table_str: &str) -> String {
    let border_chars: &[char] = &['┌', '─', '┬', '┐', '│', '├', '┼', '┤', '└', '┴', '┘'];
    let mut result = String::with_capacity(table_str.len() * 2);

    for ch in table_str.chars() {
        if border_chars.contains(&ch) {
            result.push_str("\x1b[90m");
            result.push(ch);
            result.push_str("\x1b[0m");
        } else {
            result.push(ch);
        }
    }

    result
}

fn format_model_name(model: &str) -> String {
    let name = model.strip_prefix("claude-").unwrap_or(model);
    if name.len() > 9 {
        let potential_date = &name[name.len() - 8..];
        if potential_date.chars().all(|c| c.is_ascii_digit()) && name.as_bytes()[name.len() - 9] == b'-' {
            return name[..name.len() - 9].to_string();
        }
    }
    name.to_string()
}

fn capitalize_source(source: &str) -> String {
    match source {
        "opencode" => "OpenCode".to_string(),
        "claude" => "Claude".to_string(),
        "codex" => "Codex".to_string(),
        "cursor" => "Cursor".to_string(),
        "gemini" => "Gemini".to_string(),
        "amp" => "Amp".to_string(),
        "droid" => "Droid".to_string(),
        "openclaw" => "openclaw".to_string(),
        "pi" => "Pi".to_string(),
        other => other.to_string(),
    }
}

fn run_sources_command(json: bool) -> Result<()> {
    use tokscale_core::{parse_local_sources, LocalParseOptions};

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

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
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    let headless_roots = get_headless_roots(&home_dir);
    let headless_codex_count = parsed
        .messages
        .iter()
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
            sessions_path: home_dir
                .join(".local/share/opencode/storage/message")
                .to_string_lossy()
                .to_string(),
            sessions_path_exists: home_dir
                .join(".local/share/opencode/storage/message")
                .exists(),
            legacy_paths: vec![],
            message_count: parsed.opencode_count,
            headless_supported: false,
            headless_paths: vec![],
            headless_message_count: 0,
        },
        SourceRow {
            source: "claude".to_string(),
            label: "Claude Code".to_string(),
            sessions_path: home_dir
                .join(".claude/projects")
                .to_string_lossy()
                .to_string(),
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
            sessions_path: get_codex_home(&home_dir)
                .join("sessions")
                .to_string_lossy()
                .to_string(),
            sessions_path_exists: get_codex_home(&home_dir).join("sessions").exists(),
            legacy_paths: vec![],
            message_count: parsed.codex_count,
            headless_supported: true,
            headless_paths: headless_roots
                .iter()
                .map(|root| {
                    let path = root.join("codex");
                    HeadlessPath {
                        path: path.to_string_lossy().to_string(),
                        exists: path.exists(),
                    }
                })
                .collect(),
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
            sessions_path: home_dir
                .join(".config/tokscale/cursor-cache")
                .to_string_lossy()
                .to_string(),
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
            sessions_path: home_dir
                .join(".local/share/amp/threads")
                .to_string_lossy()
                .to_string(),
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
            sessions_path: home_dir
                .join(".factory/sessions")
                .to_string_lossy()
                .to_string(),
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
            sessions_path: home_dir
                .join(".openclaw/agents")
                .to_string_lossy()
                .to_string(),
            sessions_path_exists: home_dir.join(".openclaw/agents").exists(),
            legacy_paths: vec![
                LegacyPath {
                    path: home_dir
                        .join(".clawdbot/agents")
                        .to_string_lossy()
                        .to_string(),
                    exists: home_dir.join(".clawdbot/agents").exists(),
                },
                LegacyPath {
                    path: home_dir
                        .join(".moltbot/agents")
                        .to_string_lossy()
                        .to_string(),
                    exists: home_dir.join(".moltbot/agents").exists(),
                },
                LegacyPath {
                    path: home_dir
                        .join(".moldbot/agents")
                        .to_string_lossy()
                        .to_string(),
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
            sessions_path: home_dir
                .join(".pi/agent/sessions")
                .to_string_lossy()
                .to_string(),
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
            headless_roots: headless_roots
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            sources,
            note: "Headless capture is supported for Codex CLI only.".to_string(),
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        use colored::Colorize;

        println!("\n  {}", "Local sources & session counts".cyan());
        println!(
            "  {}",
            format!(
                "Headless roots: {}",
                headless_roots
                    .iter()
                    .map(|p| p.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .bright_black()
        );
        println!();

        for row in sources {
            println!("  {}", row.label.white());
            println!(
                "  {}",
                format!(
                    "sessions: {}",
                    describe_path(&row.sessions_path, row.sessions_path_exists)
                )
                .bright_black()
            );

            if !row.legacy_paths.is_empty() {
                let legacy_desc: Vec<String> = row
                    .legacy_paths
                    .iter()
                    .map(|lp| describe_path(&lp.path, lp.exists))
                    .collect();
                println!(
                    "  {}",
                    format!("legacy: {}", legacy_desc.join(", ")).bright_black()
                );
            }

            if row.headless_supported {
                let headless_desc: Vec<String> = row
                    .headless_paths
                    .iter()
                    .map(|hp| describe_path(&hp.path, hp.exists))
                    .collect();
                println!(
                    "  {}",
                    format!("headless: {}", headless_desc.join(", ")).bright_black()
                );
                println!(
                    "  {}",
                    format!(
                        "messages: {} (headless: {})",
                        format_number(row.message_count),
                        format_number(row.headless_message_count)
                    )
                    .bright_black()
                );
            } else {
                println!(
                    "  {}",
                    format!("messages: {}", format_number(row.message_count)).bright_black()
                );
            }

            println!();
        }

        println!(
            "  {}",
            "Note: Headless capture is supported for Codex CLI only.".bright_black()
        );
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
    rt.block_on(async { auth::login().await })
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

    println!(
        "{}",
        "  Please consider starring tokscale on GitHub!".bright_black()
    );
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
            println!(
                "{}",
                "  Failed to star via gh CLI. Continuing to submit...".yellow()
            );
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
    use std::time::Instant;
    use tokscale_core::{generate_graph, ReportOptions};

    let show_progress = output.is_some() && !no_spinner;
    let include_cursor = sources
        .as_ref()
        .map_or(true, |s| s.iter().any(|src| src == "cursor"));
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
    let graph_result = rt
        .block_on(async {
            generate_graph(ReportOptions {
                home_dir: None,
                sources,
                since,
                until,
                year,
            })
            .await
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    let processing_time_ms = start.elapsed().as_millis() as u32;
    let output_data = to_ts_token_contribution_data(&graph_result);
    let json_output = serde_json::to_string_pretty(&output_data)?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, json_output)?;

        eprintln!(
            "{}",
            format!("✓ Graph data written to {}", output_path).green()
        );
        eprintln!(
            "{}",
            format!(
                "  {} days, {} sources, {} models",
                output_data.contributions.len(),
                output_data.summary.sources.len(),
                output_data.summary.models.len()
            )
            .bright_black()
        );
        eprintln!(
            "{}",
            format!(
                "  Total: {}",
                format_currency(output_data.summary.total_cost)
            )
            .bright_black()
        );

        if benchmark {
            eprintln!(
                "{}",
                format!("  Processing time: {}ms (Rust native)", processing_time_ms).bright_black()
            );
            if let Some(sync) = cursor_sync_result {
                if sync.synced {
                    eprintln!(
                        "{}",
                        format!(
                            "  Cursor: {} usage events synced (full lifetime data)",
                            sync.rows
                        )
                        .bright_black()
                    );
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

    let include_cursor = sources
        .as_ref()
        .map_or(true, |s| s.iter().any(|src| src == "cursor"));
    let has_cursor_cache = cursor::has_cursor_usage_cache();
    if include_cursor && cursor::is_cursor_logged_in() {
        println!("{}", "  Syncing Cursor usage data...".bright_black());
        let rt_sync = Runtime::new()?;
        let sync_result = rt_sync.block_on(async { cursor::sync_cursor_cache().await });
        if sync_result.synced {
            println!(
                "{}",
                format!("  Cursor: {} usage events synced", sync_result.rows).bright_black()
            );
        } else if let Some(err) = sync_result.error {
            if has_cursor_cache {
                println!(
                    "{}",
                    format!("  Cursor sync failed; using cached data: {}", err).yellow()
                );
            }
        }
    }

    println!("{}", "  Scanning local session data...".bright_black());

    let rt = Runtime::new()?;
    let graph_result = rt
        .block_on(async {
            generate_graph(ReportOptions {
                home_dir: None,
                sources,
                since,
                until,
                year,
            })
            .await
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", "  Data to submit:".white());
    println!(
        "{}",
        format!(
            "    Date range: {} to {}",
            graph_result.meta.date_range_start, graph_result.meta.date_range_end,
        )
        .bright_black()
    );
    println!(
        "{}",
        format!("    Active days: {}", graph_result.summary.active_days).bright_black()
    );
    println!(
        "{}",
        format!(
            "    Total tokens: {}",
            format_tokens_with_commas(graph_result.summary.total_tokens)
        )
        .bright_black()
    );
    println!(
        "{}",
        format!(
            "    Total cost: {}",
            format_currency(graph_result.summary.total_cost)
        )
        .bright_black()
    );
    println!(
        "{}",
        format!("    Sources: {}", graph_result.summary.sources.join(", ")).bright_black()
    );
    println!(
        "{}",
        format!("    Models: {} models", graph_result.summary.models.len()).bright_black()
    );
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
            let body: SubmitResponse =
                rt.block_on(async { resp.json().await })
                    .unwrap_or_else(|_| SubmitResponse {
                        submission_id: None,
                        username: None,
                        metrics: None,
                        warnings: None,
                        error: Some(format!(
                            "Server returned {} with unparseable response",
                            status
                        )),
                        details: None,
                    });

            if !status.is_success() {
                eprintln!(
                    "\n  {}",
                    format!(
                        "Error: {}",
                        body.error
                            .unwrap_or_else(|| "Submission failed".to_string())
                    )
                    .red()
                );
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
                    println!(
                        "{}",
                        format!("    Total tokens: {}", format_tokens_with_commas(tokens))
                            .bright_black()
                    );
                }
                if let Some(cost) = metrics.total_cost {
                    println!(
                        "{}",
                        format!("    Total cost: {}", format_currency(cost)).bright_black()
                    );
                }
                if let Some(days) = metrics.active_days {
                    println!("{}", format!("    Active days: {}", days).bright_black());
                }
            }
            println!();
            println!(
                "{}",
                format!(
                    "  View your profile: {}/u/{}",
                    api_url, credentials.username
                )
                .cyan()
            );
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
    use chrono::Utc;
    use std::io::{Read, Write};
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use uuid::Uuid;

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

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let headless_roots = get_headless_roots(&home_dir);

    let output_path = if let Some(custom_output) = output {
        let parent = Path::new(&custom_output)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        custom_output
    } else {
        let root = headless_roots
            .first()
            .cloned()
            .unwrap_or_else(|| home_dir.join(".config/tokscale/headless"));
        let dir = root.join(&source_lower);
        std::fs::create_dir_all(&dir)?;

        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H-%M-%S-%3fZ").to_string();
        let uuid_short = Uuid::new_v4()
            .to_string()
            .replace("-", "")
            .chars()
            .take(8)
            .collect::<String>();
        let filename = format!(
            "{}-{}-{}.{}",
            source_lower, timestamp, uuid_short, resolved_format
        );

        dir.join(filename).to_string_lossy().to_string()
    };

    let settings = tui::settings::Settings::load();
    let timeout = settings.get_native_timeout();

    use colored::Colorize;
    println!("\n  {}", "Headless capture".cyan());
    println!("  {}", format!("source: {}", source_lower).bright_black());
    println!("  {}", format!("output: {}", output_path).bright_black());
    println!(
        "  {}",
        format!("timeout: {}s", timeout.as_secs()).bright_black()
    );
    println!();

    let mut child = Command::new(&source_lower)
        .args(&final_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn '{}': {}", source_lower, e))?;

    let stdout = child
        .stdout
        .take()
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

    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = [0; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                output_file
                    .write_all(&buffer[..n])
                    .map_err(|e| anyhow::anyhow!("Failed to write to output file: {}", e))?;
            }
            Err(e) => {
                if timed_out.load(Ordering::SeqCst) {
                    break;
                }
                return Err(anyhow::anyhow!(
                    "Failed to read from subprocess stdout: {}",
                    e
                ));
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for subprocess: {}", e))?;

    timed_out.store(true, Ordering::SeqCst);
    let _ = timeout_handle.join();

    if timed_out.load(Ordering::SeqCst) && !status.success() {
        eprintln!(
            "{}",
            format!("\n  Subprocess timed out after {}s", timeout.as_secs()).red()
        );
        eprintln!("{}", "  Partial output saved. Increase timeout with TOKSCALE_NATIVE_TIMEOUT_MS or settings.json".bright_black());
        println!();
        std::process::exit(124);
    }

    let exit_code = status.code().unwrap_or(1);

    println!(
        "{}",
        format!("✓ Saved headless output to {}", output_path).green()
    );
    println!();

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
