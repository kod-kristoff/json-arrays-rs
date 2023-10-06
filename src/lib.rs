pub mod error;
pub mod json;
pub mod writer;

use std::path::Path;

pub use crate::error::Error;
pub use crate::writer::{Writer, WriterBuilder};

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = String>, Error> {
    json::load_from_file(path)
}
