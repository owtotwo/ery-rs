use clap::Parser;
use ery::app::App;
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

    let search_text: Option<&str> = cli.text.as_deref();

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    let mut app = App::with_sender(tui.sender.clone());
    if let Some(text) = search_text {
        tui.set_search_text(text); // set search text from start
        app.send_query(text)?; // then search it automatically
    }

    tui.run_loop(&mut app)?;

    Ok(())
}
