use crate::{
    app::App,
    ery::{Query, QueryResults},
};
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use everything_sdk::error::EverythingError;
use tui_textarea::{CursorMove, Input, Key, TextArea};

use anyhow::Result;

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
    // ignore key release for windows
    if key_event.kind == KeyEventKind::Release {
        return Ok(());
    }
    match key_event.code {
        // Quit application on `Esc`
        KeyCode::Esc => {
            app.quit();
        }
        // Quit application on `Ctrl+C`
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        // Do query on `Enter`
        KeyCode::Enter => {
            app.send_query();
        }
        // Shift focus in different widgets
        KeyCode::Tab => {
            // TODO: do nothing now, we will support the results list selection for it.
        }
        // Other handlers passthrough to tui-textarea
        _ => key_map_for_textarea(key_event.into(), &mut app.textarea),
    }
    Ok(())
}

pub fn handle_mouse_events(mouse_event: MouseEvent, app: &mut App) -> Result<()> {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::ScrollUp => {
            app.increment_counter()
        }
        MouseEventKind::Down(MouseButton::Right) | MouseEventKind::ScrollDown => {
            app.decrement_counter()
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_tick(app: &mut App) -> Result<()> {
    app.tick();
    unreachable!() // no tick for now
}

pub fn handle_refresh_event(app: &mut App) -> Result<()> {
    app.refresh += 1;
    Ok(())
}

/// Handles the everything send-query event and updates the state of [`App`].
pub fn handle_send_query(app: &mut App) -> Result<()> {
    let query_text = app.textarea.lines().first().unwrap().to_owned();
    let query = Query {
        search: query_text,
        match_path: false,
        match_case: false,
        match_whole_word: false,
        regex: false,
        max: 64, // TODO: limit for now
        offset: 0,
        sort_type: Default::default(),
        request_flags: Default::default(),
    };
    app.query_sender.send(query).unwrap();
    Ok(())
}

/// Handles the everything query back event and updates the app state
pub fn handle_query_back(results: QueryResults, app: &mut App) -> Result<()> {
    app.query_results = results;
    Ok(())
}

/// Handles the everything query back event and updates the app state
pub fn handle_query_error(_error: EverythingError, _app: &mut App) -> Result<()> {
    unreachable!() // no error event for now
}

/// Custom key mappings for [`tui_textarea::TextArea`], enjoy an good typing for input.
///
/// Ref: https://docs.rs/tui-textarea/0.4.0/tui_textarea/#define-your-own-key-mappings
fn key_map_for_textarea(input: Input, textarea: &mut TextArea) {
    match input {
        // Copy selected text
        Input {
            key: Key::Char('c'),
            ctrl: true,
            shift: false,
            alt: false,
        }
        | Input { key: Key::Copy, .. } => {
            textarea.copy();
        }
        // Cut selected text
        Input {
            key: Key::Char('x'),
            ctrl: true,
            shift: false,
            alt: false,
        }
        | Input { key: Key::Cut, .. } => {
            textarea.cut();
        }
        // Paste yanked text
        Input {
            key: Key::Char('v'),
            ctrl: true,
            shift: false,
            alt: false,
        }
        | Input {
            key: Key::Paste, ..
        } => {
            textarea.paste();
        }
        // Move cursor forward by word
        Input {
            key: Key::Right,
            ctrl: true,
            shift: false,
            alt: false,
        } => textarea.move_cursor(CursorMove::WordForward),
        // Move cursor backward by word
        Input {
            key: Key::Left,
            ctrl: true,
            shift: false,
            alt: false,
        } => textarea.move_cursor(CursorMove::WordBack),
        // Delete one character next to cursor
        Input {
            key: Key::Backspace,
            ctrl: true,
            shift: false,
            alt: false,
        } => {
            textarea.delete_word();
        }
        // Select forward by word
        Input {
            key: Key::Right,
            ctrl: true,
            shift: true,
            alt: false,
        } => {
            textarea.start_selection();
            textarea.move_cursor(CursorMove::WordForward);
        }
        // Select backward by word
        Input {
            key: Key::Left,
            ctrl: true,
            shift: true,
            alt: false,
        } => {
            textarea.start_selection();
            textarea.move_cursor(CursorMove::WordBack);
        }
        // Undo
        Input {
            key: Key::Char('z'),
            ctrl: true,
            shift: false,
            alt: false,
        } => {
            textarea.undo();
        }
        // ignore it, do nothing
        Input { ctrl: true, .. } => {}
        // will not capture in here
        Input {
            key: Key::Enter | Key::Esc | Key::Tab,
            ..
        } => {
            unreachable!()
        }
        input => {
            textarea.input(input);
        }
    }
}
