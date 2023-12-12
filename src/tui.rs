use crate::app::App;
use crate::event::EventHandler;
use crate::ui;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::panic;

use anyhow::Result;

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub event_handler: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, event_handler: EventHandler) -> Self {
        Self {
            terminal,
            event_handler,
        }
    }

    /// Initializes the TUI.
    ///
    /// get ready for TUI, enable the raw mode and set terminal props.
    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        // Use stdout instead of stderr for refresh efficiency. (I don't know why stderr is slow)
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        // deal with panic
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Ref: https://stackoverflow.com/a/73467496
            Self::reset().expect("failed to reset the terminal, double-panic now");
            panic_hook(panic_info);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Render UI with app state.
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    /// Resets the TUI, be a static helper method for exit and panic_hook.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        // It's the same here for stdout.
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the TUI.
    ///
    /// cleanup for TUI, disable the raw mode and set terminal props.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
