use std::error::Error;

use clap::Parser;
use governance_core::{models::*, send_data};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// path of the json file to parse
    #[clap(short, long)]
    path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let json_parsed = json_file(cli.path)?;

    send_data(json_parsed).await?;

    Ok(())
}
