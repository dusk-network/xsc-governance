use serde::Deserialize;

#[derive(Deserialize)]
pub struct Username {
    username: String,
    identifier: String,
    first_name: String,
    last_name: String,
}
