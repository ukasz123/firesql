use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};
use tui_textarea::TextArea;

pub(crate) fn run(
    mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
) -> io::Result<()> {
    let mut sql_textarea = TextArea::default();
    sql_textarea.set_block(Block::bordered().title("Enter SQL:"));
    loop {
        terminal.draw(|frame| {
            let greeting =
                Line::from("Welcome to FireSQL client! (press 'Esc' to quit)").light_red();

            let [title_area, main_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(frame.area());

            let [left_pane, right_pane] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Fill(1)])
                    .areas(main_area);

            frame.render_widget(greeting, title_area);
            frame.render_widget(&sql_textarea, left_pane);
            frame.render_widget(
                Block::bordered()
                    .border_style(Style::new().dark_gray())
                    .title("Firebase query"),
                right_pane,
            );
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                return Ok(());
            } else {
                sql_textarea.input(key);
            }
        }
    }
}
