mod app;
mod ui;
mod viz;

use app::{App, Popup};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{
    io::{self, stdout, Stdout},
    time::Duration,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    let tick_rate = Duration::from_millis(100);

    loop {
        app.tick = app.tick.wrapping_add(1);
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code);
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode) {
    match app.popup {
        Popup::Source => handle_source_popup(app, code),
        Popup::Seed => handle_seed_popup(app, code),
        Popup::Viz => handle_viz_popup(app, code),
        Popup::None => handle_main(app, code),
    }
}

fn handle_main(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_method > 0 {
                app.selected_method -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_method + 1 < app.methods.len() {
                app.selected_method += 1;
            }
        }
        KeyCode::Enter => app.run_selected(),
        KeyCode::Char('s') => {
            app.popup = Popup::Source;
            app.popup_selection = app.selected_source;
        }
        KeyCode::Char('S') => {
            app.popup = Popup::Seed;
        }
        KeyCode::Char('v') => {
            app.popup = Popup::Viz;
            app.popup_selection = app.selected_source;
        }
        _ => {}
    }
}

fn handle_source_popup(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => app.popup = Popup::None,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.popup_selection > 0 {
                app.popup_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.popup_selection + 1 < app.sources.len() {
                app.popup_selection += 1;
            }
        }
        KeyCode::Enter => {
            app.selected_source = app.popup_selection;
            app.popup = Popup::None;
            app.status_message = Some(format!("source set to {}", app.current_source_name()));
        }
        _ => {}
    }
}

fn handle_seed_popup(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => app.popup = Popup::None,
        KeyCode::Enter => {
            app.popup = Popup::None;
            let hint = if app.seed.is_empty() {
                "cleared".to_string()
            } else {
                format!("set to {}", app.seed)
            };
            app.status_message = Some(format!("seed {}", hint));
        }
        KeyCode::Backspace => {
            app.seed.pop();
        }
        KeyCode::Char(c) => app.seed.push(c),
        _ => {}
    }
}

fn handle_viz_popup(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('v') | KeyCode::Enter => {
            app.selected_source = app.popup_selection;
            app.popup = Popup::None;
            app.status_message =
                Some(format!("source set to {}", app.current_source_name()));
        }
        KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
            if app.popup_selection > 0 {
                app.popup_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') => {
            if app.popup_selection + 1 < app.sources.len() {
                app.popup_selection += 1;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_closes_source_popup() {
        let mut app = App::new();
        app.popup = Popup::Source;
        handle_key(&mut app, KeyCode::Esc);
        assert_eq!(app.popup, Popup::None);
    }

    #[test]
    fn esc_closes_seed_popup() {
        let mut app = App::new();
        app.popup = Popup::Seed;
        handle_key(&mut app, KeyCode::Esc);
        assert_eq!(app.popup, Popup::None);
    }

    #[test]
    fn up_navigates_methods_upward() {
        let mut app = App::new();
        handle_main(&mut app, KeyCode::Down);
        handle_main(&mut app, KeyCode::Down);
        assert_eq!(app.selected_method, 2);
        handle_main(&mut app, KeyCode::Up);
        assert_eq!(app.selected_method, 1);
    }

    #[test]
    fn up_clamps_at_top_boundary() {
        let mut app = App::new();
        assert_eq!(app.selected_method, 0);
        handle_main(&mut app, KeyCode::Up);
        assert_eq!(app.selected_method, 0, "Up at index 0 must not underflow");
    }

    #[test]
    fn up_navigates_source_popup_selection() {
        let mut app = App::new();
        app.popup = Popup::Source;
        app.popup_selection = 2;
        handle_source_popup(&mut app, KeyCode::Up);
        assert_eq!(app.popup_selection, 1);
    }

    #[test]
    fn backspace_pops_seed_character() {
        let mut app = App::new();
        app.popup = Popup::Seed;
        app.seed = "abc".to_string();
        handle_seed_popup(&mut app, KeyCode::Backspace);
        assert_eq!(app.seed, "ab");
    }

    #[test]
    fn char_appends_to_seed() {
        let mut app = App::new();
        app.popup = Popup::Seed;
        handle_seed_popup(&mut app, KeyCode::Char('4'));
        handle_seed_popup(&mut app, KeyCode::Char('2'));
        assert_eq!(app.seed, "42");
    }

    #[test]
    fn quit_key_sets_should_quit() {
        let mut app = App::new();
        handle_main(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn source_key_opens_source_popup() {
        let mut app = App::new();
        handle_main(&mut app, KeyCode::Char('s'));
        assert_eq!(app.popup, Popup::Source);
    }

    #[test]
    fn shift_s_opens_seed_popup() {
        let mut app = App::new();
        handle_main(&mut app, KeyCode::Char('S'));
        assert_eq!(app.popup, Popup::Seed);
    }
}
