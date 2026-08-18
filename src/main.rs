mod app;
mod config;
mod fetcher;
mod model;
mod opml;
mod reader;
mod sample_data;
mod storage;
mod theme;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::path::PathBuf;

use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "ratarss", version = "0.1.0", about = "A beautiful, minimal, themable, and customizable RSS reader in Ratatui in Rust resembling NetNewsWire")]
struct Cli {
    /// Import subscriptions from an OPML file
    #[arg(short, long, value_name = "FILE")]
    import: Option<PathBuf>,

    /// Export subscriptions to an OPML file
    #[arg(short, long, value_name = "FILE")]
    export: Option<PathBuf>,

    /// Add a new RSS/Atom feed URL
    #[arg(short, long, value_name = "URL")]
    add: Option<String>,

    /// Optional folder name when adding a feed with --add
    #[arg(short, long, value_name = "FOLDER")]
    folder: Option<String>,

    /// Theme to use (e.g. "NetNewsWire Dark", "Catppuccin Mocha", "Tokyo Night", "Nord", "Gruvbox Dark", "Dracula", "Minimal Monochrome")
    #[arg(short, long, value_name = "THEME")]
    theme: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let runtime_handle = tokio::runtime::Handle::current();

    // Headless CLI operations
    if let Some(ref import_path) = cli.import {
        println!("📥 Importing feeds from OPML: {}", import_path.display());
        let feeds = opml::parse_opml_file(import_path)
            .map_err(|e| format!("Failed to parse OPML: {}", e))?;
        let db = storage::Database::new()?;
        let count = feeds.len();
        for feed in feeds {
            db.add_or_update_feed(&feed)?;
        }
        println!("✅ Successfully imported {count} feeds into RataRSS database.");
        return Ok(());
    }

    if let Some(ref export_path) = cli.export {
        println!("📤 Exporting feeds to OPML: {}", export_path.display());
        let db = storage::Database::new()?;
        let feeds = db.get_all_feeds()?;
        let opml_content = opml::export_opml(&feeds, "RataRSS Subscriptions")
            .map_err(|e| format!("Failed to export OPML: {}", e))?;
        std::fs::write(export_path, opml_content)?;
        println!("✅ Successfully exported {} feeds to {}", feeds.len(), export_path.display());
        return Ok(());
    }

    if let Some(ref feed_url) = cli.add {
        println!("➕ Adding feed: {}", feed_url);
        let fetcher = fetcher::FeedFetcher::new();
        let (feed, articles) = fetcher
            .discover_or_create_feed(feed_url, cli.folder.clone())
            .await
            .map_err(|e| format!("Failed to add feed: {}", e))?;
        let db = storage::Database::new()?;
        let title = feed.title.clone();
        db.add_or_update_feed(&feed)?;
        let count = db.insert_articles(&articles)?;
        println!("✅ Successfully added '{}' with {} articles.", title, count);
        return Ok(());
    }

    // Set panic hook to restore terminal gracefully
    let original_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_panic(info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Create App state
    let mut app = App::new(runtime_handle)?;

    if let Some(theme_name) = cli.theme {
        app.theme = theme::Theme::by_name(&theme_name);
        app.config.theme = theme_name;
    }

    // Main event loop
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    while !app.should_quit {
        terminal.draw(|f| ui::render_app(&app, f))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // Only process Press events (avoid double triggers on Windows/macOS)
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        app.handle_key_event(key);
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse_event(mouse);
                }
                Event::Resize(_, _) => {
                    // Terminal resized, redrawn on next loop
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
