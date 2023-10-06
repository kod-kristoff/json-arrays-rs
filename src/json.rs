use std::{fs, io, path::Path};

use crate::Error;

use struson::reader::{JsonReader, JsonStreamReader};

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<JsonIterator, Error> {
    println!("load_from_file(path={})", path.as_ref().display());
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut reader = JsonStreamReader::new(reader);

    match reader.peek()? {}

    Ok(JsonIterator {})
}

pub struct JsonIterator {
    reader: JsonStreamReader,
}

impl Iterator for JsonIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
