use std::cmp::min;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::app::App;

// Prefer standard 8-bit RGB colors, therefore, more terminals can be supported.
// Ref: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit

// Everything (voidtools) icon color.
const _MAIN_COLOR_24_BIT: Color = Color::Rgb(255, 128, 0);
// Ref: https://stackoverflow.com/a/60392218
// RGB ff8000 -> xterm color approx 208 (DarkOrange	#ff8700	rgb(255,135,0))
const MAIN_COLOR_8_BIT: Color = Color::Indexed(208);
const MAIN_COLOR: Color = MAIN_COLOR_8_BIT;
const _FONT_COLOR_24_BIT: Color = Color::Rgb(229, 192, 123);
// RGB e5c07b -> xterm color approx 180 (d7af87)
const FONT_COLOR_8_BIT: Color = Color::Indexed(180);
const FONT_COLOR: Color = FONT_COLOR_8_BIT;
const _DARK_GRAY_COLOR: Color = Color::DarkGray;
const TERM_GRAY_COLOR: Color = Color::Indexed(8);
const GRAY_COLOR: Color = TERM_GRAY_COLOR;

#[derive(Debug)]
pub struct UI<'a> {
    pub textarea: TextArea<'a>,
    pub is_focus_search_bar: bool,
    cursor_style: Style,
    pub list_state: ListState,
    pub last_page_height: Option<u16>,
}

impl UI<'_> {
    pub fn new() -> Self {
        // let mut textarea = TextArea::new(vec!["♿😊☺".to_string()]);
        // textarea.move_cursor(CursorMove::End);
        let textarea = TextArea::new(vec![]);
        let cursor_style = textarea.cursor_style();
        let is_focus_now = true;
        let list_state = ListState::default().with_offset(0).with_selected(None);
        UI {
            textarea,
            is_focus_search_bar: is_focus_now,
            cursor_style,
            list_state,
            last_page_height: None,
        }
    }

    pub fn render(&mut self, app: &mut App, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(frame.size());

        self.last_page_height = Some(
            chunks[1]
                .inner(&Margin {
                    vertical: 1,
                    horizontal: 1,
                })
                .height,
        );

        self.textarea.set_style(Style::default().fg(FONT_COLOR));
        self.textarea.set_cursor_line_style(Style::default());
        if self.is_focus_search_bar {
            self.textarea.set_cursor_style(self.cursor_style);
        } else {
            self.textarea
                .set_cursor_style(self.textarea.cursor_line_style());
        }
        self.textarea.set_block(
            Block::default()
                .style(Style::default().fg(MAIN_COLOR))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Everything"),
        );
        let widget = self.textarea.widget();

        frame.render_widget(widget, chunks[0]);

        let results = app.query_results.read().unwrap();

        let (num, total) = (results.number, results.total);
        assert!(num <= total);

        let offset = self.list_state.offset();
        let selected = self.list_state.selected();
        let block = Block::new()
            .title(vec![
                Span::styled(
                    format!("Total Results: {total} (Offset: {offset} Selected: {selected:?})"),
                    Style::default().fg(if num > 0 { MAIN_COLOR } else { GRAY_COLOR }),
                ),
                Span::styled(
                    format!("『{}』", results.search.to_string_lossy()),
                    Style::default().fg(GRAY_COLOR),
                ),
            ])
            .style(Style::default().fg(MAIN_COLOR))
            .borders(Borders::ALL);

        let items: Vec<ListItem> = results
            .entrys
            .iter()
            .map(|entry| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        if entry.is_folder { "📁 " } else { "📄 " },
                        Style::default().fg(GRAY_COLOR),
                    ),
                    Span::styled(
                        format!("{}", entry.filename.as_ref().unwrap().to_string_lossy()),
                        Style::default().fg(FONT_COLOR),
                    ),
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        format!("{}", entry.path.as_ref().unwrap().display()),
                        Style::default().italic().fg(GRAY_COLOR),
                    ),
                ])])
            })
            .collect();

        let list = if self.is_focus_search_bar {
            List::new(items).block(block)
        } else {
            List::new(items)
                .block(block)
                .highlight_style(Style::default().fg(Color::Indexed(220)))
        };

        // let list = list;
        // .highlight_style(Style::default().underlined());
        // .highlight_style(Style::default().fg(Color::Rgb(255, 169, 0)));

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }

    pub fn set_search_text(&mut self, text: &str) {
        let old_yank = self.textarea.yank_text();
        self.textarea.set_yank_text(text);
        self.textarea.select_all();
        self.textarea.paste();
        self.textarea.set_yank_text(old_yank);
    }

    pub fn is_selected(&self) -> bool {
        self.list_state.selected().is_some()
    }

    pub fn is_first_selected(&self) -> bool {
        self.list_state.selected().is_some_and(|i| i == 0)
    }

    pub fn select_first(&mut self, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn select_last(&mut self, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                self.list_state.select(Some(results.number as usize - 1));
            }
        }
    }

    pub fn select_previous_n(&mut self, n: usize, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                let last = (results.number - 1) as usize;
                self.list_state.select(
                    self.list_state
                        .selected()
                        .and_then(|i| Some(min(last, i.saturating_sub(n)))),
                );
            }
        }
    }

    pub fn select_next_n(&mut self, n: usize, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                let last = (results.number - 1) as usize;
                self.list_state.select(
                    self.list_state
                        .selected()
                        .and_then(|i| Some(min(last, i.saturating_add(n)))),
                );
            }
        };
    }

    pub fn is_first_page(&self) -> bool {
        self.list_state.offset() == 0
    }

    pub fn is_last_page(&self, results_number: u32) -> bool {
        let page_height = self.last_page_height.unwrap() as u32;
        if results_number <= page_height {
            true
        } else {
            let offset = self.list_state.offset();
            (results_number - offset as u32) <= page_height
        }
    }

    pub fn select_next_page(&mut self, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                if self.is_last_page(results.number) {
                    self.list_state.select(Some(results.number as usize - 1));
                } else {
                    let old_offset = self.list_state.offset();
                    let page_height = self.last_page_height.unwrap() as usize;
                    let new_offset = old_offset.saturating_add(page_height);
                    *self.list_state.offset_mut() = new_offset;

                    let n = new_offset - old_offset;
                    let last = (results.number - 1) as usize;
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .and_then(|i| Some(min(last, i.saturating_add(n)))),
                    );
                }
            }
        };
    }

    pub fn select_previous_page(&mut self, app: &mut App) {
        if let Ok(results) = app.query_results.try_read() {
            if results.number > 0 {
                if self.is_first_page() {
                    self.list_state.select(Some(0));
                } else {
                    let old_offset = self.list_state.offset();
                    let page_height = self.last_page_height.unwrap() as usize;
                    let new_offset = old_offset.saturating_sub(page_height);
                    *self.list_state.offset_mut() = new_offset;

                    let n = old_offset - new_offset;
                    let last = (results.number - 1) as usize;
                    self.list_state.select(
                        self.list_state
                            .selected()
                            .and_then(|i| Some(min(last, i.saturating_sub(n)))),
                    );
                }
            }
        };
    }

    pub fn unselect(&mut self) {
        self.list_state.select(None);
    }
}

/// Custom key mappings for [`tui_textarea::TextArea`], enjoy an good typing for input.
///
/// Ref: https://docs.rs/tui-textarea/0.4.0/tui_textarea/#define-your-own-key-mappings
pub fn key_map_for_textarea(input: Input, textarea: &mut TextArea) {
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
