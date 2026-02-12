mod app;
mod cache;
pub mod config;
pub mod data;
mod event;
pub mod settings;
mod themes;
mod ui;

pub use app::{App, Tab, TuiConfig};
pub use cache::{is_cache_stale, load_cached_data, save_cached_data};
pub use data::{DataLoader, Source, UsageData};
pub use event::{Event, EventHandler};

use std::collections::HashSet;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

pub fn run(
    theme: &str,
    refresh: u64,
    debug: bool,
    sources: Option<Vec<String>>,
    since: Option<String>,
    until: Option<String>,
    year: Option<String>,
    initial_tab: Option<Tab>,
) -> Result<()> {
    if debug {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    }

    let config = TuiConfig {
        theme: theme.to_string(),
        refresh,
        sessions_path: None,
        sources: sources.clone(),
        since: since.clone(),
        until: until.clone(),
        year: year.clone(),
        initial_tab,
    };

    let mut enabled_sources = HashSet::new();
    if let Some(ref cli_sources) = sources {
        for source_str in cli_sources {
            match source_str.as_str() {
                "opencode" => enabled_sources.insert(Source::OpenCode),
                "claude" => enabled_sources.insert(Source::Claude),
                "codex" => enabled_sources.insert(Source::Codex),
                "cursor" => enabled_sources.insert(Source::Cursor),
                "gemini" => enabled_sources.insert(Source::Gemini),
                "amp" => enabled_sources.insert(Source::Amp),
                "droid" => enabled_sources.insert(Source::Droid),
                "openclaw" => enabled_sources.insert(Source::OpenClaw),
                "pi" => enabled_sources.insert(Source::Pi),
                _ => false,
            };
        }
    } else {
        for source in Source::all() {
            enabled_sources.insert(*source);
        }
    }

    let cached_data = load_cached_data(&enabled_sources);
    let cache_is_stale = cached_data.is_some() && is_cache_stale(&enabled_sources);
    let has_cached_data = cached_data.is_some();

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        let _ = disable_raw_mode();
        return Err(e.into());
    }

    let backend = CrosstermBackend::new(stdout);
    let terminal_result = Terminal::new(backend);
    let mut terminal = match terminal_result {
        Ok(t) => t,
        Err(e) => {
            restore_terminal_best_effort();
            return Err(e.into());
        }
    };

    let mut app = match App::new_with_cached_data(config, cached_data) {
        Ok(a) => a,
        Err(e) => {
            restore_terminal(&mut terminal);
            return Err(e);
        }
    };

    let (tx, rx) = mpsc::channel::<Result<UsageData>>();
    let needs_background_load = !has_cached_data || cache_is_stale;

    if needs_background_load {
        app.set_background_loading(true);

        let bg_sources: Vec<Source> = enabled_sources.iter().copied().collect();
        let bg_since = since.clone();
        let bg_until = until.clone();
        let bg_year = year.clone();
        let bg_enabled_sources = enabled_sources.clone();

        thread::spawn(move || {
            let loader = DataLoader::with_filters(None, bg_since, bg_until, bg_year);
            let result = loader.load(&bg_sources);

            if let Ok(ref data) = result {
                save_cached_data(data, &bg_enabled_sources);
            }

            let _ = tx.send(result);
        });
    }

    let mut events = EventHandler::new(Duration::from_millis(100));

    let result = run_loop_with_background(&mut terminal, &mut app, &mut events, rx);

    restore_terminal(&mut terminal);

    result
}

fn restore_terminal_best_effort() {
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = disable_raw_mode();
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();
}

fn run_loop_with_background(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
    background_rx: mpsc::Receiver<Result<UsageData>>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        match background_rx.try_recv() {
            Ok(result) => {
                app.set_background_loading(false);
                match result {
                    Ok(data) => {
                        app.update_data(data);
                        app.set_status("Data loaded");
                    }
                    Err(e) => {
                        app.set_error(Some(e.to_string()));
                        app.set_status(&format!("Error: {}", e));
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                // Only treat as error if we were still waiting for data
                // After data is received, disconnect is expected (thread completed)
                if app.background_loading {
                    app.set_background_loading(false);
                    app.set_error(Some("Background thread disconnected".to_string()));
                    app.set_status("Error: Background thread disconnected");
                }
            }
            Err(TryRecvError::Empty) => {
                // No data available yet, continue
            }
        }

        match events.next()? {
            Event::Tick => {
                app.on_tick();
            }
            Event::Key(key) => {
                if app.handle_key_event(key) {
                    break;
                }
            }
            Event::Mouse(mouse) => {
                app.handle_mouse_event(mouse);
            }
            Event::Resize(w, h) => {
                app.handle_resize(w, h);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

pub fn test_data_loading() -> Result<()> {
    println!("Testing data loading...");

    let loader = DataLoader::new(None);
    let all_sources = vec![
        Source::OpenCode,
        Source::Claude,
        Source::Cursor,
        Source::Gemini,
        Source::Codex,
        Source::Amp,
        Source::Droid,
        Source::OpenClaw,
        Source::Pi,
    ];

    let data = loader.load(&all_sources)?;

    println!("Loaded {} models", data.models.len());
    println!("Total cost: ${:.2}", data.total_cost);

    println!("\nAll models (source:model):");
    let mut models = data.models.clone();
    models.sort_by(|a, b| {
        let source_cmp = a.source.cmp(&b.source);
        if source_cmp == std::cmp::Ordering::Equal {
            a.model.cmp(&b.model)
        } else {
            source_cmp
        }
    });
    for m in &models {
        println!("{}:{}", m.source.to_lowercase(), m.model);
    }

    Ok(())
}
