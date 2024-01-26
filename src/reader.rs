// use std::{io, marker::PhantomData};

// use serde::de::DeserializeOwned;

// use crate::error::Result;

// #[derive(Debug)]
// pub struct ReaderBuilder {
//     capacity: usize,
//     is_json_lines: bool,
// }

// impl Default for ReaderBuilder {
//     fn default() -> Self {
//         ReaderBuilder {
//             capacity: 8 * (1 << 10),
//             is_json_lines: false,
//         }
//     }
// }

// impl ReaderBuilder {
//     /// Create a new builder for configuring JSON reading.
//     ///
//     /// To convert a builder into a reader, call one of the methods starting
//     /// with `from_`.
//     pub fn new() -> ReaderBuilder {
//         ReaderBuilder::default()
//     }
//     /// Build a CSV parser from this configuration that reads data from `rdr`.
//     ///
//     /// Note that the CSV reader is buffered automatically, so you should not
//     /// wrap `rdr` in a buffered reader like `io::BufReader`.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use std::error::Error;
//     /// use csv::ReaderBuilder;
//     ///
//     /// # fn main() { example().unwrap(); }
//     /// fn example() -> Result<(), Box<dyn Error>> {
//     ///     let data = "\
//     /// city,country,pop
//     /// Boston,United States,4628910
//     /// Concord,United States,42695
//     /// ";
//     ///     let mut rdr = ReaderBuilder::new().from_reader(data.as_bytes());
//     ///     for result in rdr.records() {
//     ///         let record = result?;
//     ///         println!("{:?}", record);
//     ///     }
//     ///     Ok(())
//     /// }
//     /// ```
//     pub fn from_reader<R: io::Read>(&self, rdr: R) -> Reader<R> {
//         Reader::new(self, rdr)
//     }
// }

// #[derive(Debug)]
// pub struct Reader<R> {
//     /// The underlying reader.
//     rdr: io::BufReader<R>,
// }

// impl<R: io::Read> Reader<R> {
//     /// Create a new CSV reader given a builder and a source of underlying
//     /// bytes.
//     fn new(builder: &ReaderBuilder, rdr: R) -> Reader<R> {
//         Reader {
//             // core: Box::new(builder.builder.build()),
//             rdr: io::BufReader::with_capacity(builder.capacity, rdr),
//             // state: ReaderState {
//             //     headers: None,
//             //     has_headers: builder.has_headers,
//             //     flexible: builder.flexible,
//             //     trim: builder.trim,
//             //     first_field_count: None,
//             //     cur_pos: Position::new(),
//             //     first: false,
//             //     seeked: false,
//             //     eof: ReaderEofState::NotEof,
//             // },
//         }
//     }

//     /// Create a new CSV parser with a default configuration for the given
//     /// reader.
//     ///
//     /// To customize CSV parsing, use a `ReaderBuilder`.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use std::error::Error;
//     /// use csv::Reader;
//     ///
//     /// # fn main() { example().unwrap(); }
//     /// fn example() -> Result<(), Box<dyn Error>> {
//     ///     let data = "\
//     /// city,country,pop
//     /// Boston,United States,4628910
//     /// Concord,United States,42695
//     /// ";
//     ///     let mut rdr = Reader::from_reader(data.as_bytes());
//     ///     for result in rdr.records() {
//     ///         let record = result?;
//     ///         println!("{:?}", record);
//     ///     }
//     ///     Ok(())
//     /// }
//     /// ```
//     pub fn from_reader(rdr: R) -> Reader<R> {
//         ReaderBuilder::new().from_reader(rdr)
//     }

//     /// Returns a borrowed iterator over deserialized records.
//     ///
//     /// Each item yielded by this iterator is a `Result<D, Error>`.
//     /// Therefore, in order to access the record, callers must handle the
//     /// possibility of error (typically with `try!` or `?`).
//     ///
//     /// If `has_headers` was enabled via a `ReaderBuilder` (which is the
//     /// default), then this does not include the first record. Additionally,
//     /// if `has_headers` is enabled, then deserializing into a struct will
//     /// automatically align the values in each row to the fields of a struct
//     /// based on the header row.
//     ///
//     /// # Example
//     ///
//     /// This shows how to deserialize CSV data into normal Rust structs. The
//     /// fields of the header row are used to match up the values in each row
//     /// to the fields of the struct.
//     ///
//     /// ```
//     /// use std::error::Error;
//     ///
//     /// #[derive(Debug, serde::Deserialize, Eq, PartialEq)]
//     /// struct Row {
//     ///     city: String,
//     ///     country: String,
//     ///     #[serde(rename = "popcount")]
//     ///     population: u64,
//     /// }
//     ///
//     /// # fn main() { example().unwrap(); }
//     /// fn example() -> Result<(), Box<dyn Error>> {
//     ///     let data = "\
//     /// city,country,popcount
//     /// Boston,United States,4628910
//     /// ";
//     ///     let mut rdr = csv::Reader::from_reader(data.as_bytes());
//     ///     let mut iter = rdr.deserialize();
//     ///
//     ///     if let Some(result) = iter.next() {
//     ///         let record: Row = result?;
//     ///         assert_eq!(record, Row {
//     ///             city: "Boston".to_string(),
//     ///             country: "United States".to_string(),
//     ///             population: 4628910,
//     ///         });
//     ///         Ok(())
//     ///     } else {
//     ///         Err(From::from("expected at least one record but got none"))
//     ///     }
//     /// }
//     /// ```
//     ///
//     /// # Rules
//     ///
//     /// For the most part, any Rust type that maps straight-forwardly to a CSV
//     /// record is supported. This includes maps, structs, tuples and tuple
//     /// structs. Other Rust types, such as `Vec`s, arrays, and enums have
//     /// a more complicated story. In general, when working with CSV data, one
//     /// should avoid *nested sequences* as much as possible.
//     ///
//     /// Maps, structs, tuples and tuple structs map to CSV records in a simple
//     /// way. Tuples and tuple structs decode their fields in the order that
//     /// they are defined. Structs will do the same only if `has_headers` has
//     /// been disabled using [`ReaderBuilder`](struct.ReaderBuilder.html),
//     /// otherwise, structs and maps are deserialized based on the fields
//     /// defined in the header row. (If there is no header row, then
//     /// deserializing into a map will result in an error.)
//     ///
//     /// Nested sequences are supported in a limited capacity. Namely, they
//     /// are flattened. As a result, it's often useful to use a `Vec` to capture
//     /// a "tail" of fields in a record:
//     ///
//     /// ```
//     /// use std::error::Error;
//     ///
//     /// #[derive(Debug, serde::Deserialize, Eq, PartialEq)]
//     /// struct Row {
//     ///     label: String,
//     ///     values: Vec<i32>,
//     /// }
//     ///
//     /// # fn main() { example().unwrap(); }
//     /// fn example() -> Result<(), Box<dyn Error>> {
//     ///     let data = "foo,1,2,3";
//     ///     let mut rdr = csv::ReaderBuilder::new()
//     ///         .has_headers(false)
//     ///         .from_reader(data.as_bytes());
//     ///     let mut iter = rdr.deserialize();
//     ///
//     ///     if let Some(result) = iter.next() {
//     ///         let record: Row = result?;
//     ///         assert_eq!(record, Row {
//     ///             label: "foo".to_string(),
//     ///             values: vec![1, 2, 3],
//     ///         });
//     ///         Ok(())
//     ///     } else {
//     ///         Err(From::from("expected at least one record but got none"))
//     ///     }
//     /// }
//     /// ```
//     ///
//     /// In the above example, adding another field to the `Row` struct after
//     /// the `values` field will result in a deserialization error. This is
//     /// because the deserializer doesn't know when to stop reading fields
//     /// into the `values` vector, so it will consume the rest of the fields in
//     /// the record leaving none left over for the additional field.
//     ///
//     /// Finally, simple enums in Rust can be deserialized as well. Namely,
//     /// enums must either be variants with no arguments or variants with a
//     /// single argument. Variants with no arguments are deserialized based on
//     /// which variant name the field matches. Variants with a single argument
//     /// are deserialized based on which variant can store the data. The latter
//     /// is only supported when using "untagged" enum deserialization. The
//     /// following example shows both forms in action:
//     ///
//     /// ```
//     /// use std::error::Error;
//     ///
//     /// #[derive(Debug, serde::Deserialize, PartialEq)]
//     /// struct Row {
//     ///     label: Label,
//     ///     value: Number,
//     /// }
//     ///
//     /// #[derive(Debug, serde::Deserialize, PartialEq)]
//     /// #[serde(rename_all = "lowercase")]
//     /// enum Label {
//     ///     Celsius,
//     ///     Fahrenheit,
//     /// }
//     ///
//     /// #[derive(Debug, serde::Deserialize, PartialEq)]
//     /// #[serde(untagged)]
//     /// enum Number {
//     ///     Integer(i64),
//     ///     Float(f64),
//     /// }
//     ///
//     /// # fn main() { example().unwrap(); }
//     /// fn example() -> Result<(), Box<dyn Error>> {
//     ///     let data = "\
//     /// label,value
//     /// celsius,22.2222
//     /// fahrenheit,72
//     /// ";
//     ///     let mut rdr = csv::Reader::from_reader(data.as_bytes());
//     ///     let mut iter = rdr.deserialize();
//     ///
//     ///     // Read the first record.
//     ///     if let Some(result) = iter.next() {
//     ///         let record: Row = result?;
//     ///         assert_eq!(record, Row {
//     ///             label: Label::Celsius,
//     ///             value: Number::Float(22.2222),
//     ///         });
//     ///     } else {
//     ///         return Err(From::from(
//     ///             "expected at least two records but got none"));
//     ///     }
//     ///
//     ///     // Read the second record.
//     ///     if let Some(result) = iter.next() {
//     ///         let record: Row = result?;
//     ///         assert_eq!(record, Row {
//     ///             label: Label::Fahrenheit,
//     ///             value: Number::Integer(72),
//     ///         });
//     ///         Ok(())
//     ///     } else {
//     ///         Err(From::from(
//     ///             "expected at least two records but got only one"))
//     ///     }
//     /// }
//     /// ```
//     pub fn deserialize<D>(&mut self) -> DeserializeRecordsIter<R, D>
//     where
//         D: DeserializeOwned,
//     {
//         DeserializeRecordsIter::new(self)
//     }
// }

// /// A borrowed iterator over deserialized records.
// ///
// /// The lifetime parameter `'r` refers to the lifetime of the underlying
// /// CSV `Reader`. The type parameter `R` refers to the underlying `io::Read`
// /// type, and `D` refers to the type that this iterator will deserialize a
// /// record into.
// pub struct DeserializeRecordsIter<'r, R: 'r, D> {
//     rdr: &'r mut Reader<R>,
//     obj: BytesObject,
//     // rec: StringRecord,
//     // headers: Option<StringRecord>,
//     _priv: PhantomData<D>,
// }

// impl<'r, R: io::Read, D: DeserializeOwned> DeserializeRecordsIter<'r, R, D> {
//     fn new(rdr: &'r mut Reader<R>) -> DeserializeRecordsIter<'r, R, D> {
//         // let headers = if !rdr.state.has_headers {
//         //     None
//         // } else {
//         //     rdr.headers().ok().map(Clone::clone)
//         // };
//         DeserializeRecordsIter {
//             rdr,
//             // rec: StringRecord::new(),
//             obj: BytesObject::new(),
//             // headers,
//             _priv: PhantomData,
//         }
//     }

//     /// Return a reference to the underlying CSV reader.
//     pub fn reader(&self) -> &Reader<R> {
//         &self.rdr
//     }

//     /// Return a mutable reference to the underlying CSV reader.
//     pub fn reader_mut(&mut self) -> &mut Reader<R> {
//         &mut self.rdr
//     }
// }

// impl<'r, R: io::Read, D: DeserializeOwned> Iterator for DeserializeRecordsIter<'r, R, D> {
//     type Item = Result<D>;

//     fn next(&mut self) -> Option<Result<D>> {
//         match self.rdr.read_object(&mut self.obj) {
//             Err(err) => Some(Err(err)),
//             Ok(false) => None,
//             Ok(true) => Some(self.obj.deserialize()),
//         }
//     }
// }
// #[cfg(test)]
// mod tests {

//     use std::error::Error;

//     use crate::reader::{Reader, ReaderBuilder};
//     use serde::Deserialize;

//     #[derive(Debug, Deserialize, PartialEq)]
//     struct Row {
//         city: String,
//         country: String,
//         // Serde allows us to name our headers exactly,
//         // even if they don't match our struct field names.
//         #[serde(rename = "popcount")]
//         population: u64,
//     }
//     const JSON_DATA: &str = r#"[{"city":"Boston","country":"United States","popcount":4628910},{"city":"Concord","country":"United States","popcount":42695}]"#;
//     #[test]
//     fn example_json() -> Result<(), Box<dyn Error>> {
//         let mut rdr = Reader::from_reader(JSON_DATA);
//         let mut data = Vec::new();
//         let mut iter = rdr.deserialize();
//         for result in iter {
//             let row: Row = result?;
//             data.push(row);
//         }

//         // let data = String::from_utf8(wtr.into_inner()?)?;
//         assert_eq!(
//             data,
//             vec![
//                 Row {
//                     city: "Boston".to_string(),
//                     country: "United States".to_string(),
//                     population: 4628910,
//                 },
//                 Row {
//                     city: "Concord".to_string(),
//                     country: "United States".to_string(),
//                     population: 42695,
//                 }
//             ]
//         );
//         Ok(())
//     }

//     // #[test]
//     // fn example_json_lines() -> Result<(), Box<dyn Error>> {
//     //     let mut wtr = WriterBuilder::new().json_lines(true).from_writer(vec![]);
//     //     wtr.serialize(Row {
//     //         city: "Boston",
//     //         country: "United States",
//     //         population: 4628910,
//     //     })?;
//     //     wtr.serialize(Row {
//     //         city: "Concord",
//     //         country: "United States",
//     //         population: 42695,
//     //     })?;

//     //     let data = String::from_utf8(wtr.into_inner()?)?;
//     //     assert_eq!(
//     //         data,
//     //         "{\"city\":\"Boston\",\"country\":\"United States\",\"popcount\":4628910}\n{\"city\":\"Concord\",\"country\":\"United States\",\"popcount\":42695}"
//     //     );
//     //     Ok(())
//     // }
// }
