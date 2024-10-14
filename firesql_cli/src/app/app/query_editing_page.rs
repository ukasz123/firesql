use std::fmt::Debug;

use ratatui::{
    crossterm::event::{self},
    layout::{Constraint, Layout},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::*,
};

use tui_textarea::*;

pub struct QueryEditingState<'a> {
    sql_textarea: TextArea<'a>,
    firebase_query: Option<String>,
}

impl QueryEditingState<'_> {
    pub(crate) fn new() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_style(Style::default().remove_modifier(Modifier::all()));
        QueryEditingState {
            sql_textarea: text_area,
            firebase_query: None,
        }
    }

    pub(crate) fn handle_events(&mut self, event: &event::Event) -> Result<(), std::io::Error> {
        if let event::Event::Key(key) = event {
            if self.sql_textarea.input(*key) {
                self.firebase_query = Some(self.sql_textarea.lines().join("\n"))
            }
        }
        Ok(())
    }
}

impl<'a> Widget for &QueryEditingState<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let greeting = Line::from("Welcome to FireSQL client! (press 'Esc' to quit)").light_red();

        let [title_area, main_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);

        let [left_pane, right_pane] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Fill(1)]).areas(main_area);

        greeting.render(title_area, buf);

        let sql_block = Block::bordered().title("Enter SQL:");
        let sql_area = sql_block.inner(left_pane);
        sql_block.render(left_pane, buf);
        self.sql_textarea.render(sql_area, buf);

        let firebase_query: &str = self.firebase_query.as_deref().unwrap_or("");
        Paragraph::new(firebase_query)
            .block(
                Block::bordered()
                    .border_style(Style::new().dark_gray())
                    .title("Firebase query"),
            )
            .render(right_pane, buf);
    }
}

impl Debug for QueryEditingState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "QueryEditingState({:?} -> {:?}), ",
            self.sql_textarea.lines(),
            self.firebase_query,
        ))
    }
}
