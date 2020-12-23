use crossterm::{
    execute, write_ansi_code, queue,
    terminal::{self, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, ClearType, SetTitle },
    event::{self, Event as CEvent, KeyEvent, KeyCode, KeyModifiers},
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
    None,
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
                Msg::Key(ev) => {
                    use KeyCode::*;
                    if ev.modifiers.eq(&KeyModifiers::NONE) {
                        match ev.code {
                            Char('q') => {
                                Self::quit(term);
                                break;
                            },
                            Char('j') | Down => {
                                self.move_to(Directions::Down);
                            },
                            Char('k') | Up  => {
                                self.move_to(Directions::Up);
                            },
                            Char('h') | Left  => {
                                self.move_to(Directions::Left);
                            },
                            Char('l') | Right  => {
                                self.move_to(Directions::Right);
                            },
                            Tab => { self.next_tab(); }
                            Home => { self.library(); }
                            Char('s') => { self.settings(); }
                            Char('n') | PageDown  => {self.next_tab();}
                            Char('p') | PageUp  => {self.prev_tab();}
                            Char(c) => {self.char(c);  },
                            _ => {}
                        };
                    } else if ev.modifiers.contains(KeyModifiers::CONTROL) {
                        match ev.code {
                            Char('c') | Down => {
                                Self::quit(term);
                                break;
                            },
                            Char('j') | Down => { self.next_tab(); },
                            Char('k') | Up => { self.prev_tab(); },
                            Char('f')  => match self.current {
                                CurrentView::Library => self.search_library(),
                                CurrentView::File(_) => self.find_on_page(),
                                _ => {},
                            },
                            _ => {}
                        }


                    } else if ev.modifiers.contains(KeyModifiers::SHIFT) {
                        match ev.code {
                            Char('j') | Down => { self.next_tab(); },
                            Char('k') | Down => { self.prev_tab(); },
                            _ => {}

                        }
                    }
                },
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
        match self.current {
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
            CurrentView::None => { return Ok(()) }
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

    pub fn library(&mut self) -> () {
        self.current = CurrentView::Library;
    }

    pub fn settings(&mut self) -> () {
        self.current = CurrentView::Settings;
    }

    pub fn next_tab(&mut self) -> CurrentView {
        let curr = match self.current {
            CurrentView::File(idx) => {
                if idx == self.open.len() as u8 {
                    self.current = CurrentView::Library;
                    return CurrentView::Library;
                } else  {
                    self.current = CurrentView::File(idx+1);
                    return CurrentView::File(idx+1);
                }
            },
            CurrentView::Browse => {
                self.current = CurrentView::Library;
                return CurrentView::Library;
            }
            CurrentView::Library => {
                self.current = CurrentView::File(0);
                return CurrentView::File(0);
            }
            CurrentView::Settings => {
                self.current = CurrentView::Library;
                return CurrentView::Library;
            }
            _ => { self.current = CurrentView::Library; return CurrentView::Library; }
        };
    }

    pub fn prev_tab(&mut self) -> CurrentView {
        let curr = match self.current {
            CurrentView::File(idx) => {
                if idx == 0 as u8 {
                    self.current = CurrentView::Library;
                    return CurrentView::Library;
                } else  {
                    self.current = CurrentView::File(idx-1);
                    return CurrentView::File(idx-1);
                }
            },
            CurrentView::Browse => {
                self.current = CurrentView::File(self.open.len() as u8);
                return CurrentView::File(self.open.len() as u8);
            }
            CurrentView::Library => {
                self.current = CurrentView::Browse;
                return CurrentView::Browse;
            }
            CurrentView::Settings => {
                self.current = CurrentView::Library;
                return CurrentView::Library;
            }
            _ => { self.current = CurrentView::Library; return CurrentView::Library; }
        };
    }

    pub fn move_to(&mut self, dir: Directions) -> () {

    }

    pub fn tick(&mut self) -> () {

    }

    pub fn char(&mut self, ch: char) -> () {}

    pub fn search_library(&self) {}

    pub fn find_on_page(&self) {}

    pub fn quit(mut term: Terminal<CrosstermBackend<Stdout>>) -> Result<(), crate::error::EError> {
        terminal::disable_raw_mode()?;
        execute!(
            term.backend_mut(),
            LeaveAlternateScreen,
        )?;
        term.show_cursor()?;
        Ok(())
    }
}

pub enum Msg {
    Key(KeyEvent),
    Tick,
    Quit,
}

pub enum Directions {
    Up, Down, Left, Right
}
