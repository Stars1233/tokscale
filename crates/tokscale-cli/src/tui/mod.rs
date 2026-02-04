mod app;
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

pub fn run(theme: &str, refresh: u64, debug: bool) -> Result<()> {
    if debug {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = TuiConfig {
        theme: theme.to_string(),
        refresh,
        sessions_path: None,
    };
    let mut app = App::new(config)?;

    app.load_data()?;

    let mut events = EventHandler::new(Duration::from_millis(100));

    let result = run_loop(&mut terminal, &mut app, &mut events);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
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
