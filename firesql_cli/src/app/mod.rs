pub(crate) mod app;

use std::io;

pub(crate) fn run_app(
    mut app: app::App,
    mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| frame.render_widget(&app, frame.area()))?;
        match app.handle_events()? {
            app::EventResult::Quit => return Ok(()),
            app::EventResult::Continue => {}
        };
    }
}
