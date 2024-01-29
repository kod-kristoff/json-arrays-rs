#[allow(unused_imports)]
use crate::error::{Error, Result};
use serde::Serialize;
use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
};

#[derive(Debug)]
pub struct WriterBuilder {
    is_json_lines: bool,
}

impl Default for WriterBuilder {
    fn default() -> Self {
        WriterBuilder {
            is_json_lines: false,
        }
    }
}

impl WriterBuilder {
    /// Create a new builder for configuring JSON writing.
    ///
    /// To convert a builder into a writer, call one of the methods starting
    /// with `from_`.
    pub fn new() -> WriterBuilder {
        WriterBuilder::default()
    }
    pub fn from_writer<W: io::Write>(&self, wtr: W) -> Writer<W> {
        Writer::new(self, wtr)
    }

    /// Build a JSON writer from this configuration that writes data to the
    /// given file path. The file is truncated if it already exists.
    ///
    /// If there was a problem opening the file at the given path, then this
    /// returns the corresponding error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use json_arrays::WriterBuilder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut wtr = WriterBuilder::new().from_path("foo.csv")?;
    ///     wtr.serialize("a")?;
    ///     wtr.serialize("x")?;
    ///     wtr.flush()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path<P: AsRef<Path>>(&self, path: P) -> Result<Writer<BufWriter<File>>> {
        Ok(Writer::new(self, BufWriter::new(File::create(path)?)))
    }

    /// Whether to write in json lines format.
    ///
    /// This is disabled by default.
    ///
    /// # Example
    ///
    /// ```
    /// use std::error::Error;
    ///
    /// use json_arrays::WriterBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Row<'a> {
    ///     city: &'a str,
    ///     country: &'a str,
    ///     // Serde allows us to name our headers exactly,
    ///     // even if they don't match our struct field names.
    ///     #[serde(rename = "popcount")]
    ///     population: u64,
    /// }
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut wtr = WriterBuilder::new()
    ///                    .json_lines(true)
    ///                    .from_writer(vec![]);
    ///     wtr.serialize(Row {
    ///         city: "Boston",
    ///         country: "United States",
    ///         population: 4628910,
    ///     })?;
    ///     wtr.serialize(Row {
    ///         city: "Concord",
    ///         country: "United States",
    ///         population: 42695,
    ///     })?;
    ///
    ///     let data = String::from_utf8(wtr.into_inner()?)?;
    ///     assert_eq!(data, "\
    /// {\"city\":\"Boston\",\"country\":\"United States\",\"popcount\":4628910}\n\
    /// {\"city\":\"Concord\",\"country\":\"United States\",\"popcount\":42695}\
    /// ");
    ///     Ok(())
    /// }
    /// ```
    pub fn json_lines(&mut self, yes: bool) -> &mut WriterBuilder {
        self.is_json_lines = yes;
        self
    }
}
#[derive(Debug)]
pub struct Writer<W: io::Write> {
    wtr: Option<W>,
    state: WriterState,
}

#[derive(Debug)]
struct WriterState {
    array_start: ArrayState,
    array_end: ArrayState,
    delimiter: DelimiterState,
    delimiter_token: [u8; 1],
    panicked: bool,
}

#[derive(Debug, Clone, Copy)]
enum ArrayState {
    Write,
    DidWrite,
    DidNotWrite,
    None,
}
#[derive(Debug, Clone, Copy)]
enum DelimiterState {
    Write,
    WriteNext,
}

impl<W: io::Write> Drop for Writer<W> {
    fn drop(&mut self) {
        if self.wtr.is_some() && !self.state.panicked {
            let _ = self.close();
        }
    }
}

impl Writer<File> {
    /// Build a JSON writer with a default configuration that writes data to the
    /// given file path. The file is truncated if it already exists.
    ///
    /// If there was a problem opening the file at the given path, then this
    /// returns the corresponding error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use json_arrays::Writer;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut wtr = Writer::from_path("foo.json")?;
    ///     wtr.serialize("a")?;
    ///     wtr.serialize("x")?;
    ///     wtr.close()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<BufWriter<File>>> {
        WriterBuilder::new().from_path(path)
    }
}

impl<W: io::Write> Writer<W> {
    fn new(builder: &WriterBuilder, wtr: W) -> Writer<W> {
        let array_state = if builder.is_json_lines {
            ArrayState::None
        } else {
            ArrayState::Write
        };

        let delimiter_token = if builder.is_json_lines {
            [b'\n']
        } else {
            [b',']
        };
        Writer {
            wtr: Some(wtr),
            state: WriterState {
                array_start: array_state,
                array_end: array_state,
                delimiter: DelimiterState::WriteNext,
                delimiter_token,
                panicked: false,
            },
        }
    }
    pub fn from_writer(wtr: W) -> Writer<W> {
        WriterBuilder::new().from_writer(wtr)
    }
    /// Serialize a single record using Serde.
    ///
    /// # Example
    ///
    /// This shows how to serialize normal Rust structs as JSON records. The
    /// fields of the struct are used to write a header row automatically.
    /// (Writing the header row automatically can be disabled by building the
    /// CSV writer with a [`WriterBuilder`](struct.WriterBuilder.html) and
    /// calling the `has_headers` method.)
    ///
    /// ```
    /// use std::error::Error;
    ///
    /// use json_arrays::writer::Writer;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Row<'a> {
    ///     city: &'a str,
    ///     country: &'a str,
    ///     // Serde allows us to name our headers exactly,
    ///     // even if they don't match our struct field names.
    ///     #[serde(rename = "popcount")]
    ///     population: u64,
    /// }
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut wtr = Writer::from_writer(vec![]);
    ///     wtr.serialize(Row {
    ///         city: "Boston",
    ///         country: "United States",
    ///         population: 4628910,
    ///     })?;
    ///     wtr.serialize(Row {
    ///         city: "Concord",
    ///         country: "United States",
    ///         population: 42695,
    ///     })?;
    ///
    ///     let data = String::from_utf8(wtr.into_inner()?)?;
    ///     assert_eq!(data, "[\
    /// {\"city\":\"Boston\",\"country\":\"United States\",\"popcount\":4628910},\
    /// {\"city\":\"Concord\",\"country\":\"United States\",\"popcount\":42695}\
    /// ]");
    ///     Ok(())
    /// }
    /// ```
    pub fn serialize<S: Serialize>(&mut self, record: S) -> Result<()> {
        if let ArrayState::Write = self.state.array_start {
            let wrote_array_start = self.write_array_start();
            if wrote_array_start {
                self.state.array_start = ArrayState::DidWrite;
            } else {
                self.state.array_start = ArrayState::DidNotWrite;
            }
        }
        match self.state.delimiter {
            DelimiterState::Write => self.write_delimiter()?,
            DelimiterState::WriteNext => self.state.delimiter = DelimiterState::Write,
        };
        // self.write_terminator()?;
        serde_json::to_writer(self.wtr.as_mut().unwrap(), &record).unwrap();
        // if let HeaderState::Write = self.state.header {
        //     let wrote_header = serialize_header(self, &record)?;
        //     if wrote_header {
        //         self.write_terminator()?;
        //         self.state.header = HeaderState::DidWrite;
        //     } else {
        //         self.state.header = HeaderState::DidNotWrite;
        //     };
        // }
        // serialize(self, &record)?;
        // self.write_terminator()?;
        Ok(())
    }

    fn write_array_start(&mut self) -> bool {
        match self.wtr.as_mut().unwrap().write(b"[") {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn write_array_end(&mut self) -> bool {
        match self.wtr.as_mut().unwrap().write(b"]") {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn write_delimiter(&mut self) -> Result<()> {
        self.wtr
            .as_mut()
            .unwrap()
            .write(&self.state.delimiter_token)
            .unwrap();
        Ok(())
    }
    pub fn flush(&mut self) -> io::Result<()> {
        // self.flush_buf()?;
        self.wtr.as_mut().unwrap().flush()?;
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        if let ArrayState::Write = self.state.array_end {
            let wrote_array_end = self.write_array_end();
            if wrote_array_end {
                self.state.array_end = ArrayState::DidWrite;
            } else {
                self.state.array_end = ArrayState::DidNotWrite;
            }
        }
        self.flush()?;
        Ok(())
    }
    pub fn into_inner(mut self) -> Result<W> {
        match self.close() {
            Ok(()) => Ok(self.wtr.take().unwrap()),
            Err(err) => todo!("handle err {:?}", err),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use crate::writer::{Writer, WriterBuilder};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Row<'a> {
        city: &'a str,
        country: &'a str,
        // Serde allows us to name our headers exactly,
        // even if they don't match our struct field names.
        #[serde(rename = "popcount")]
        population: u64,
    }

    #[test]
    fn example_json() -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.serialize(Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        })?;
        wtr.serialize(Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        })?;

        let data = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(
            data,
            r#"[{"city":"Boston","country":"United States","popcount":4628910},{"city":"Concord","country":"United States","popcount":42695}]"#
        );
        Ok(())
    }

    #[test]
    fn example_json_lines() -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new().json_lines(true).from_writer(vec![]);
        wtr.serialize(Row {
            city: "Boston",
            country: "United States",
            population: 4628910,
        })?;
        wtr.serialize(Row {
            city: "Concord",
            country: "United States",
            population: 42695,
        })?;

        let data = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(
            data,
            "{\"city\":\"Boston\",\"country\":\"United States\",\"popcount\":4628910}\n{\"city\":\"Concord\",\"country\":\"United States\",\"popcount\":42695}"
        );
        Ok(())
    }
}
