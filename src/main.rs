mod app;
mod data;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::{io, time::Duration};

use app::{App, Focus, InputMode};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let res = run(&mut terminal);
    ratatui::restore();
    res
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    loop {
        app.poll_worker();
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                match app.input_mode {
                    InputMode::Insert => match key.code {
                        KeyCode::Enter => app.trigger_search(),
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Backspace => {
                            app.query.pop();
                        }
                        KeyCode::Char(c) => app.query.push(c),
                        _ => {}
                    },
                    InputMode::Normal => match key.code {
                        _ if key.modifiers.contains(KeyModifiers::CONTROL) => {}
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('/') | KeyCode::Char('i') => {
                            app.input_mode = InputMode::Insert;
                        }
                        KeyCode::Tab => app.toggle_focus(),
                        KeyCode::Left | KeyCode::Char('h') => app.focus = Focus::Results,
                        KeyCode::Right | KeyCode::Char('l') => app.focus = Focus::Preview,
                        KeyCode::Down | KeyCode::Char('j') => match app.focus {
                            Focus::Results => app.next_row(),
                            Focus::Preview => app.scroll_preview(5),
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.focus {
                            Focus::Results => app.prev_row(),
                            Focus::Preview => app.scroll_preview(-5),
                        },
                        KeyCode::Char('y') => app.copy_path_to_clipboard(),
                        KeyCode::Char('m') => app.mirror_exploit(),
                        _ => {}
                    },
                }
            }
        }
    }
}
