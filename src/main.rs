#![allow(dead_code)]

use std::io::Write;

use clap::{Parser, Subcommand};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Pull {
        #[arg(short, long, help="Source username or email")]
        username: String,

        #[arg(short, long, help="Source instance")]
        instance: Url,
    },
}


enum Origin {
    Source,
    Destination,
}

fn get_password(origin: Origin) -> String {
    let key = match origin {
        Origin::Source => "LEMMY_SRC_PW",
        Origin::Destination => "LEMMY_DST_PW",
    };

    match std::env::var(key) {
        Ok(pw) => pw,
        Err(_) => {
            print!("Password({}): ", key);
            std::io::stdout().flush().unwrap();
            let pw = rpassword::read_password().unwrap();
            pw
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Pull { username, instance }) => {
            let pw = get_password(Origin::Source);
        },
        None => {}
    }
}
