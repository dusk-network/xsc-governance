use std::error::Error;

use clap::{Parser, Subcommand};
use governance_core::{models::*, send_call, CsvParser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// CSV Data to parse
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Parse the activites list in the csv file
    Activity {
        /// Path of the csv file
        #[clap(short, long)]
        path: String,
    },
    /// Parse the white list in the csv file
    Whitelist {
        /// Path of the csv file
        #[clap(short, long)]
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    match &cli.command {
        Command::Activity { path } => {
            let activities: Vec<Activity> = CsvParser::from_path(path)?
                .headers(vec!["sender", "buyer", "amount", "timestamp"])
                .parse()?;

            send_call(|caller, signature| ActivityCall {
                caller,
                signature,
                activities: activities.iter().flat_map(<[u8; 144]>::from).collect(),
                count: 0,
            })
            .await?;
        }
        Command::Whitelist { path } => {
            let whitelist: Vec<Whitelist> = CsvParser::from_path(path)?
                .headers(vec!["add", "remove"])
                .parse_enum()?;

            send_call(|caller, signature| WhitelistCall {
                caller,
                signature,
                whitelist: whitelist.iter().flat_map(<[u8; 144]>::from).collect(),
                count: 0,
            })
            .await?
        }
    };

    Ok(())
}
