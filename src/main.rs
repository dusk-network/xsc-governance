mod models;

use crate::models::Username;
use csv::StringRecord;

use csv::Reader;

fn main() {
    let mut usernames = Reader::from_path("username.csv").expect("Cannot read file");

    let mut vec: Vec<Username> = Vec::new();

    let header = StringRecord::from(vec!["username", "identifier", "first_name", "last_name"]);

    for result in usernames.records() {
        let record = result.unwrap();
        println!("{:?}", record);
        let username: Username = record.deserialize(Some(&header)).unwrap();
        vec.push(username);
    }
}
