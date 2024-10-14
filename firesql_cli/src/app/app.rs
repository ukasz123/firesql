use std::{cell::RefCell, io};

use query_editing_page::QueryEditingState;
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    widgets::Widget,
};

mod query_editing_page;

#[derive(Debug)]
pub(crate) enum CurrentScreen {
    QueryEditing(RefCell<QueryEditingState<'static>>),
    QueryResults,
}

pub struct App {
    current_page: CurrentScreen,
}

impl App {
    pub(crate) fn new() -> Self {
        App {
            current_page: CurrentScreen::QueryEditing(RefCell::new(QueryEditingState::new())),
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match &self.current_page {
            CurrentScreen::QueryEditing(editing_state) => {
                editing_state.borrow().render(area, buf);
            }
            CurrentScreen::QueryResults => todo!(),
        }
    }
}

impl App {
    pub(crate) fn handle_events(&mut self) -> io::Result<EventResult> {
        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                return Ok(EventResult::Quit);
            }
        }
        match &self.current_page {
            CurrentScreen::QueryEditing(editing_state) => {
                editing_state.borrow_mut().handle_events(&event)
            }
            CurrentScreen::QueryResults => todo!(),
        }?;
        Ok(EventResult::Continue)
    }
}

pub(crate) enum EventResult {
    Quit,
    Continue,
}
