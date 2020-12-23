use crossterm::{
    execute, write_ansi_code, queue,
    terminal::{self, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, ClearType, SetTitle },
    event::{self, Event as CEvent, KeyEvent, KeyCode},
    cursor::{self, Hide, Show},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Layout, Rect, Direction, Constraint, Alignment,},
    terminal::{Terminal, TerminalOptions, Frame, Viewport},
    text::{Text, Span, Spans, StyledGrapheme},
    buffer::{Buffer, Cell,},
    widgets::{Tabs, Row, Paragraph, Widget, Block, Borders, Wrap},
    style::{Style, Color, Modifier,},
};
use std::{
    io::{self, Read, Write, Stdout},
    borrow::Cow,
    path::{PathBuf, Path},
    fs, thread
};
use chrono::{DateTime, Local, Duration};
use crate::store::Store;

pub struct App<'a> {
    layout: Layout,
    current: CurrentView,
    open: Vec<std::path::PathBuf>,
    pub tabs: tui::widgets::Tabs<'a>,
    store: Store,
}

pub enum CurrentView {
    Library,
    File(u8),
    Settings,
    Browse,
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
        let tabs = Tabs::new(vec![Spans(vec![lib])])
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .select(0)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            )
            .block(Block::default().borders(Borders::ALL).title("Tabs"));
        Self {
            tabs,
            layout: Layout::default(),
            open: Vec::new(),
            current: CurrentView::Library,
            store: Store::default(),
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
        let chunks = Self::chunks(f.size());
        let tabs = self.tabs.clone();
        f.render_widget(tabs, chunks[0]);
        let block = Block::default()
            .style(Style::default()
                .bg(Color::Black)
                .fg(Color::LightGreen));

        let block = |title: String| {
            Block::default().borders(Borders::ALL)
                .style(Style::default().bg(Color::Black).fg(Color::White))
                .title(Span::styled(title, Style::default().add_modifier(Modifier::BOLD)))
        };
        let content = match self.current {
            CurrentView::File(idx) => {
                let path: &Path = self.open[idx as usize].as_path();
                let file = fs::read_to_string(path)?;
                let pars = file.split_terminator("\n\n")
                    .map(|s| {
                        let p = Paragraph::new(s)
                            .style(Style::default())
                            .block(Block::default()
                                .borders(Borders::NONE))
                            .alignment(Alignment::Left)
                            .scroll((0, 0))
                            .wrap(Wrap { trim: false });
                        f.render_widget(p.clone(), chunks[1]);
                        return p;
                    })
                    .collect::<Vec<Paragraph>>();
                let spans = file.lines().into_iter().map(|ln| {
                    Spans::from(Span::styled(ln, Style::default()
                        .fg(Color::White).bg(Color::Black)))
                }).collect::<Vec<Spans>>();
            }
            CurrentView::Library => {
                for item in self.store.clone().history {
                    let file = fs::read_to_string(item.as_path())?;
                    let itm = item.clone().to_owned();
                    let block = block(itm.to_str().unwrap_or(file.lines().next().unwrap_or_default()).to_string());
                    f.render_widget(block, chunks[1]);
                }
                let spans = Spans::from(Span::styled("Library",
                    Style::default().fg(Color::White).bg(Color::Black)));
            },
            CurrentView::Settings => {},
            CurrentView::Browse => {}
        };
        let status = Block::default().borders(Borders::ALL);
        f.render_widget(status, chunks[2]);
        Ok(())
    }

    pub fn chunks(area: Rect) -> Vec<Rect> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(8),
                Constraint::Percentage(10),
                Constraint::Percentage(8),
            ])
            .split(area);
        chunks
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
