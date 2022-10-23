use clap::{Parser, Subcommand};

use governance_core::models::*;
use governance_core::CsvParser;

use csv::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Name of the person to greet
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Activity {
        #[clap(short, long)]
        path: String,
    },
    Whitelist {
        #[clap(short, long)]
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Args::parse();

    match &cli.command {
        Command::Activity { path } => {
            let usernames: Vec<Activity> = CsvParser::from_path(path)?
                .headers(vec!["sender", "buyer", "amount", "timestamp"])
                .parse()?;

            println!("{:?}", usernames);
        }
        Command::Whitelist { path } => {
            let ty: Vec<Whitelist> = CsvParser::from_path(path)?
                .headers(vec!["Add", "Remove"])
                .parse_enum()?;
        }
    };

    Ok(())
}
