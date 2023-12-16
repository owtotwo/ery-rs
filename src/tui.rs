mod ui;

use crate::app::App;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, KeyModifiers, MouseButton,
    MouseEventKind,
};
use crossterm::event::{KeyEvent, MouseEvent};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::panic;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};

use crossterm::event::{self, Event as CrosstermEvent};
use tui_textarea::CursorMove;

use anyhow::Result;

#[derive(Debug)]
pub struct Tui<'a, B: Backend> {
    terminal: Terminal<B>,
    is_running: bool,
    pub sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    ui: ui::UI<'a>,
}

#[derive(Debug)]
pub enum Event {
    /// App refresh request.
    Refresh,
    /// Key press/release/repeat.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

impl<B: Backend> Tui<'_, B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            terminal,
            is_running: false,
            sender: tx,
            receiver: rx,
            ui: ui::UI::new(),
        }
    }

    pub fn run_loop(&mut self, app: &mut App) -> Result<()> {
        self.init()?;

        self.term()?;

        self.is_running = true;
        while self.is_running() {
            // Render the user interface.
            self.draw(app)?;
            // Handle events.
            match self.receiver.recv()? {
                Event::Refresh => self.handle_refresh_event(app)?,
                Event::Key(key_event) => self.handle_key_events(key_event, app)?,
                Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event, app)?,
                Event::Resize(_, _) => {}
            }
        }

        self.exit()?;
        Ok(())
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

    // run crossterm event loop to capture user input, and send it to the tui.
    pub fn term(&mut self) -> Result<()> {
        const TICK_RATE: Duration = Duration::from_millis(250);
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = TICK_RATE
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(TICK_RATE);

                if event::poll(timeout).expect("failed to poll events") {
                    match event::read().expect("failed to read the event") {
                        CrosstermEvent::FocusGained => Ok(()),
                        CrosstermEvent::FocusLost => Ok(()),
                        CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                        CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                        CrosstermEvent::Paste(_) => Ok(()),
                        CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    }
                    .expect("failed to send terminal event")
                }

                if last_tick.elapsed() >= TICK_RATE {
                    // it seems that we may not need the tick, just do nothing when user do nothing
                    // sender.send(Event::Tick).expect("failed to send tick event");
                    last_tick = Instant::now();
                }
            }
        });
        Ok(())
    }

    /// Render UI with app state.
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| self.ui.render(app, frame))?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    fn quit(&mut self) {
        self.is_running = false;
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

    pub fn set_search_text(&mut self, text: &str) {
        self.ui.set_search_text(text);
    }

    pub fn handle_refresh_event(&mut self, _app: &mut App) -> Result<()> {
        Ok(())
    }

    pub fn handle_mouse_events(&mut self, mouse_event: MouseEvent, _app: &mut App) -> Result<()> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::ScrollUp => {
                self.ui.textarea.move_cursor(CursorMove::Back);
            }
            MouseEventKind::Down(MouseButton::Right) | MouseEventKind::ScrollDown => {
                self.ui.textarea.move_cursor(CursorMove::Forward);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent, app: &mut App) -> Result<()> {
        // ignore key release for windows
        if key_event.kind == KeyEventKind::Release {
            return Ok(());
        }
        match key_event.code {
            // Quit application on `Esc`
            KeyCode::Esc => {
                self.quit();
            }
            // Quit application on `Ctrl+C`
            KeyCode::Char('c') | KeyCode::Char('C')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.quit();
            }
            // Do query on `Enter`
            KeyCode::Enter => {
                app.send_query(self.ui.textarea.lines()[0].as_str())?;
            }
            // Shift focus in different widgets
            KeyCode::Tab => {
                // TODO: do nothing now, we will support the results list selection for it.
            }
            // Other handlers passthrough to tui-textarea
            _ => ui::key_map_for_textarea(key_event.into(), &mut self.ui.textarea),
        }
        Ok(())
    }
}
