pub mod library;

use library::LibraryCmd;

use std::{io, fs, path::PathBuf, process};
use crate::error::EError;

#[derive(Default)]
pub struct Cmd {
    pub open: Vec<PathBuf>,
    pub library: Vec<PathBuf>,
    debug: bool,
    help: bool,
    pub cmd: Subcmd,
}

pub enum Subcmd {
    Library(LibraryCmd),
}

impl Default for Subcmd {
    fn default() -> Self {
        Self::Library(library::LibraryCmd::default())
    }
}

impl Cmd {

    pub fn new(files: Vec<PathBuf>) ->  Self {
        Self { open: files, ..Default::default() }
    }

    pub fn run(self) -> Result<(), EError> {
        crate::app::App::new().ev_loop()
    }

    pub fn from_args() -> Result<Self, EError> {
        let args: Vec<String> = std::env::args().collect();
        let mut app = Self::default();
        let mut flagged = false;
        //while let Some((n, arg)) = args.iter().enumerate().next() {
        for (n, arg) in args.iter().enumerate() {
            println!("{} {}", n, arg);
            if flagged { flagged = false; continue }
            if arg.starts_with("-") {
                let (l, flag) = match arg.starts_with("--") {
                    true => (2, arg.split_at(2).1),
                    false => (1, arg.split_at(1).1),
                };
                match (l, flag) {
                    (1, "F") | (2, "file") => {
                        if let Some(file) = args.get(n+1) {
                            if file.ends_with(".epub") {
                                let path = std::path::PathBuf::from(file);
                                if path.exists() && path.is_file() {
                                    flagged = true;
                                    let _file = std::fs::read_to_string(file)?;
                                    app.open.push(path);
                                } else {
                                    println!("Invalid epub file");
                                    process::exit(0);
                                }
                            } else {
                                println!("Not an epub file");
                                process::exit(0);
                            }
                        }
                    },
                    (1, "D") | (2, "debug") => {
                        println!("Debug mode on")
                    },
                    (1, "H") | (2, "help") => {
                        println!("Help msg goes here")
                    },
                    (_, _) => {}
                };
            } else {
                match arg.as_str() {
                    "library" | "lib" => {
                    },
                    _ => continue,
                }
            }
        };
        Ok(app)
    }
}
