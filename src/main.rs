mod csv_parser;
mod models;

use crate::csv_parser::CsvParser;
use crate::models::Data;
use csv::Result;

fn main() -> Result<()> {
    let usernames: Vec<Data> = CsvParser::from_path("username.csv")?
        .headers(vec!["city", "country", "pop_count"])
        .parse()?;

    assert_eq!(usernames[0].city, "Boston");

    Ok(())
}
