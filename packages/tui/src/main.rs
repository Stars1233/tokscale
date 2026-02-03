mod app;
mod data;
mod event;
mod settings;
mod themes;
mod ui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::App;
use crate::data::{DataLoader, Source, UsageData};
use crate::event::{Event, EventHandler};

/// CLI arguments for tokscale-tui
#[derive(Parser, Debug)]
#[command(name = "tokscale-tui")]
#[command(author, version, about = "Terminal UI for AI token usage analytics")]
pub struct Args {
    /// Path to sessions directory
    #[arg(short, long)]
    pub sessions_path: Option<String>,

    /// Initial theme (green, halloween, teal, blue, pink, purple, orange, monochrome, ylgnbu)
    #[arg(short, long, default_value = "blue")]
    pub theme: String,

    /// Auto-refresh interval in seconds (0 to disable)
    #[arg(short, long, default_value = "0")]
    pub refresh: u64,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,

    /// Test data loading only (don't start TUI)
    #[arg(long)]
    pub test_data: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::EnvFilter::new("debug"))
            .init();
    }

    if args.test_data {
        return test_data_loading();
    }

    if args.debug {
        eprintln!("DEBUG: Starting TUI...");
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args)?;
    let event_handler = EventHandler::new(Duration::from_millis(100));
    let result = run_app(&mut terminal, &mut app, event_handler);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}

fn test_data_loading() -> Result<()> {
    println!("Testing data loader...\n");

    let loader = DataLoader::new(None);
    let all_sources: Vec<Source> = Source::all().to_vec();

    println!("Enabled sources: {:?}\n", all_sources);

    match loader.load(&all_sources) {
        Ok(data) => {
            println!("Loaded data:");
            println!("  Models: {}", data.models.len());
            println!("  Daily entries: {}", data.daily.len());
            println!("  Total tokens: {}", data.total_tokens);
            println!("  Total cost: ${:.2}", data.total_cost);
            println!("  Current streak: {} days", data.current_streak);
            println!("  Longest streak: {} days\n", data.longest_streak);

            println!("Top 20 models by cost:");
            let mut models = data.models.clone();
            models.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());
            for (i, m) in models.iter().take(20).enumerate() {
                println!(
                    "  {}. {} ({}) - {} tokens, ${:.4}",
                    i + 1,
                    m.model,
                    m.source,
                    m.tokens.total(),
                    m.cost
                );
            }

            println!("\nSample daily data with models:");
            let mut daily = data.daily.clone();
            daily.sort_by(|a, b| b.date.cmp(&a.date));
            for d in daily.iter().take(5) {
                println!(
                    "  {} - {} tokens, {} models",
                    d.date,
                    d.tokens.total(),
                    d.models.len(),
                );
                for (model_name, model_info) in d.models.iter().take(3) {
                    println!(
                        "    - {} (source: {}) - {} tokens",
                        model_name,
                        model_info.source,
                        model_info.tokens.total()
                    );
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    app.data.loading = true;

    let (tx, rx) = mpsc::channel::<Result<UsageData, String>>();
    let sources: Vec<Source> = app.enabled_sources.iter().copied().collect();
    let sessions_path = app.data_loader.sessions_path().map(|p| p.to_path_buf());

    thread::spawn(move || {
        let loader = DataLoader::new(sessions_path);
        let result = loader.load(&sources).map_err(|e| e.to_string());
        let _ = tx.send(result);
    });

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if app.data.loading {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(data) => {
                        app.data = data;
                        app.set_status("Data loaded");
                    }
                    Err(e) => {
                        app.data.loading = false;
                        app.data.error = Some(e.clone());
                        app.set_status(&format!("Error: {}", e));
                    }
                }
            }
        }

        match event_handler.next()? {
            Event::Tick => {
                app.on_tick();
            }
            Event::Key(key_event) => {
                if app.handle_key_event(key_event) {
                    break;
                }
            }
            Event::Mouse(mouse_event) => {
                app.handle_mouse_event(mouse_event);
            }
            Event::Resize(width, height) => {
                app.handle_resize(width, height);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
