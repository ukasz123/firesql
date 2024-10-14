use std::io;

mod app;

fn main() -> io::Result<()> {
    println!("Hello, world!");
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let a = app::app::App::new();
    // app::run(terminal)?;
    app::run_app(a, terminal)?;
    ratatui::restore();
    Ok(())
}
