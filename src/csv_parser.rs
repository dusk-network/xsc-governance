use std::path::PathBuf;
use std::{fs::File, io, io::Read};

use csv::{Reader, Result, StringRecord, Trim};
use serde::Deserialize;

pub struct CsvParser<T> {
    headers: Option<StringRecord>,
    data: Reader<T>,
}

impl CsvParser<File> {
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Result<Self> {
        let data = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(path.into())?;

        Ok(Self {
            headers: None,
            data,
        })
    }
}

impl<T: Read> CsvParser<T> {
    pub fn headers(mut self, headers: Vec<&str>) -> Self {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(Trim::All)
            .from_reader(self.data.into_inner());

        self.headers = Some(StringRecord::from(headers));
        self.data = reader;
        self
    }

    pub fn parse<D: for<'a> Deserialize<'a>>(mut self) -> Result<Vec<D>> {
        if let Some(headers) = self.headers {
            self.data
                .records()
                .flat_map(|record| record.map(|record| record.deserialize(Some(&headers))))
                .collect()
        } else {
            self.data.deserialize().collect()
        }
    }

    pub fn parse_enum<D>(mut self) -> Result<Vec<D>>
    where
        D: TryFrom<StringRecord, Error = io::Error>,
    {
        self.data
            .records()
            .flat_map(|record| record.map(|record| D::try_from(record).map_err(csv::Error::from)))
            .collect()
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
