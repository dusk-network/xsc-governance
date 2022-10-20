mod csv_parser;
mod models;

use crate::csv_parser::CsvParser;
use crate::models::*;

use csv::Result;

fn main() -> Result<()> {
    let usernames: Vec<Activity> = CsvParser::from_path("activity.csv")?
        .headers(vec!["sender", "buyer", "amount", "timestamp"])
        .parse()?;

    let ty: Vec<Whitelist> = CsvParser::from_path("whitelist.csv")?
        .headers(vec!["Add", "Remove"])
        .parse_enum()?;

    println!("{:?}", ty);

    Ok(())
}
