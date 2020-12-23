use std::{
    path::PathBuf,
    collections::HashMap,
};

#[derive(Default)]
pub struct Store {
    pub history: Vec<PathBuf>,
    pub positions: HashMap<u8, u16>,
}
