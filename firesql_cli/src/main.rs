use std::io;

mod app;

fn main() -> io::Result<()> {
    println!("Hello, world!");
    let mut terminal = ratatui::init();
    terminal.clear()?;
    app::run(terminal)?;
    ratatui::restore();
    Ok(())
}
