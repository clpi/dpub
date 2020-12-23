pub mod app;
pub mod cmd;
pub mod error;
pub mod store;

use std::io;
use error::EError;

pub fn run() -> Result<(), EError> {
    cmd::Cmd::from_args()?.run()
}
