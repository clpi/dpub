use crossterm::{
    execute, write_ansi_code, queue,
    terminal::{self, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, ClearType, SetTitle },
    event::{self, Event as CEvent, KeyEvent, KeyCode},
    cursor::{self, Hide, Show},
};
use tui::{
    backend::CrosstermBackend,
    terminal::{Terminal, TerminalOptions, Frame, Viewport},
    text::{Text, Span, Spans, StyledGrapheme},
    buffer::{Buffer, Cell},
    widgets::{Tabs, Row, Paragraph, Widget},
    style::{Style, Color, Modifier,},
};
use std::{
    io::{self, Read, Write, Stdout},
    borrow::Cow,
    fs, thread
};
use chrono::{DateTime, Local, Duration};

pub struct App<'a> {
    main: tui::widgets::Tabs<'a>,
}

impl<'a> App<'a> {

    pub fn new() -> App<'a> {
        let lib = Span {
            content: Cow::Borrowed("Library"),
            style: Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD)
        };
        Self {
            main: Tabs::new(vec![Spans(vec![lib])]),
        }
    }

    pub fn ev_loop(&mut self) -> Result<(), crate::error::EError> {
        let mut so = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(&so, SetTitle("epc"), EnableLineWrap, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut term = Terminal::new(backend)?;
        let (tx, rx) = std::sync::mpsc::channel();
        let tick = Duration::milliseconds(10);
        let begin = Local::now();
        thread::spawn(move || {
            let mut last = Local::now();
            loop {
                let to = tick.checked_sub(&last.signed_duration_since(begin))
                    .unwrap_or(Duration::zero());
                if let Ok(_ev) = event::poll(to.to_std().unwrap_or_default()) {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        tx.send(Msg::Key(key)).unwrap();
                    }
                }
                let now = Local::now();
                if now.signed_duration_since(now) >= tick {
                    tx.send(Msg::Tick).unwrap();
                    last = now;
                }
            }
        });
        term.clear()?;
        loop {
            term.draw(|f| { self.draw(f).unwrap(); } )?;
            use Directions::*;
            match rx.recv().unwrap() {
                Msg::Key(ev) => match ev.code {
                    KeyCode::Char('q') => {
                        terminal::disable_raw_mode()?;
                        execute!(
                            term.backend_mut(),
                            LeaveAlternateScreen,
                        )?;
                        term.show_cursor()?;
                        break;
                    },
                    KeyCode::Char('j') | KeyCode::Down => self.move_to(Directions::Down),
                    KeyCode::Char('k') | KeyCode::Up  => self.move_to(Up),
                    KeyCode::Char('h') | KeyCode::Left  => self.move_to(Left),
                    KeyCode::Char('l') | KeyCode::Right  => self.move_to(Right),
                    KeyCode::Char(c) => self.char(c),
                    _ => {}
                }
                Msg::Tick => self.tick(),
                Msg::Quit => break,
                _ => break,
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) -> Result<(), crate::error::EError> {
        Ok(())
    }

    pub fn move_to(&mut self, dir: Directions) -> () {

    }

    pub fn tick(&mut self) -> () {

    }

    pub fn char(&mut self, ch: char) -> () {}
}

pub enum Msg {
    Key(KeyEvent),
    Tick,
    Quit,
}

pub enum Directions {
    Up, Down, Left, Right
}
