use clap::Parser;
use ery::app::App;
use ery::event::{Event, EventHandler};
use ery::handler::{
    handle_key_events, handle_mouse_events, handle_query_back, handle_query_error,
    handle_refresh_event, handle_send_query, handle_tick,
};
use ery::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// search text for Everything
    text: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let search_text = cli.text.as_deref();

    run_tui(search_text)
}

fn run_tui(search_text: Option<&str>) -> anyhow::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let event_handler = EventHandler::with_tick(250);

    let mut app = App::with_sender(event_handler.sender.clone());

    if let Some(text) = search_text {
        app.set_search_text(text); // set search text from start
        handle_send_query(&mut app)?; // then search it automatically
    }

    let mut tui = Tui::new(terminal, event_handler);
    tui.init()?;

    while app.is_running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.event_handler.next()? {
            Event::Refresh => handle_refresh_event(&mut app)?,
            Event::Tick => handle_tick(&mut app)?,
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(mouse_event) => handle_mouse_events(mouse_event, &mut app)?,
            Event::Resize(_, _) => {}
            Event::SendQuery => handle_send_query(&mut app)?,
            Event::QueryBack(results) => handle_query_back(results, &mut app)?,
            Event::QueryError(error) => handle_query_error(error, &mut app)?,
        }
    }

    tui.exit()?;
    Ok(())
}
