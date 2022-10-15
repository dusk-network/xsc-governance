use std::path::PathBuf;
use std::{fs::File, io::Read};

use csv::{Reader, Result, StringRecord};
use serde::Deserialize;

pub struct CsvParser<T> {
    headers: Option<StringRecord>,
    data: Reader<T>,
}

impl CsvParser<File> {
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Result<Self> {
        let data = Reader::from_path(path.into())?;

        Ok(Self {
            headers: None,
            data,
        })
    }
}

impl<T: Read> CsvParser<T> {
    pub fn headers(mut self, headers: Vec<&str>) -> Self {
        self.headers = Some(StringRecord::from(headers));
        self
    }

    pub fn parse<D: for<'a> Deserialize<'a>>(mut self) -> Result<Vec<D>> {
        self.data.deserialize().collect()
    }
}

impl<'a> From<&'a str> for CsvParser<&'a [u8]> {
    fn from(val: &'a str) -> Self {
        Self {
            headers: None,
            data: Reader::from_reader(val.as_bytes()),
        }
    }
}
