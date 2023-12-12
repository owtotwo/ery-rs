use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

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

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.size());

    app.textarea.set_style(Style::default().fg(FONT_COLOR));
    app.textarea.set_cursor_line_style(Style::default());
    app.textarea.set_block(
        Block::default()
            .style(Style::default().fg(MAIN_COLOR))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Everything"),
    );
    let widget = app.textarea.widget();

    frame.render_widget(widget, chunks[0]);

    let tui_show_max_len = chunks[1].height;
    let results = &app.query_results;

    let (num, total) = (results.number, results.total);
    assert!(num <= total);

    let block = Block::new()
        .title(vec![
            Span::styled(
                format!("Total Results: {total}"),
                Style::default().fg(if num > 0 { MAIN_COLOR } else { GRAY_COLOR }),
            ),
            Span::styled(
                format!("『{}』", results.search.to_string_lossy()),
                Style::default().fg(GRAY_COLOR),
            ),
        ])
        .style(Style::default().fg(MAIN_COLOR))
        .borders(Borders::ALL);

    let lines: Vec<Line> = results
        .entrys
        .iter()
        .take(tui_show_max_len as usize)
        .map(|entry| {
            Line::from(vec![
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
            ])
        })
        .collect();
    let text: Text<'_> = Text::from(lines);

    let paragraph = Paragraph::new(text).block(block).style(Style::default());

    frame.render_widget(paragraph, chunks[1]);
}
