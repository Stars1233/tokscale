mod app;
pub mod config;
pub mod data;
mod event;
mod settings;
mod themes;
mod ui;

pub use app::{App, TuiConfig};
pub use data::{DataLoader, Source};
pub use event::{Event, EventHandler};

use std::io;
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
) -> Result<()> {
    if debug {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    }

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

    let config = TuiConfig {
        theme: theme.to_string(),
        refresh,
        sessions_path: None,
        sources,
        since,
        until,
        year,
    };
    let app_result = App::new(config);
    let mut app = match app_result {
        Ok(a) => a,
        Err(e) => {
            restore_terminal(&mut terminal);
            return Err(e);
        }
    };

    if let Err(e) = app.load_data() {
        restore_terminal(&mut terminal);
        return Err(e);
    }

    let mut events = EventHandler::new(Duration::from_millis(100));

    let result = run_loop(&mut terminal, &mut app, &mut events);

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

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

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
