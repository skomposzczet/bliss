#![allow(dead_code)]

mod user;
mod api;
mod profile;
mod bliss;

use std::io::Write;
use bliss::{pull, Error};
use clap::{Parser, Subcommand};
use url::Url;
use user::User;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull account settings to local profile
    Pull {
        #[arg(short, long, help="Source username or email")]
        username: String,

        #[arg(short, long, help="Source instance")]
        instance: Url,

        #[arg(short, long, help="Local profile name")]
        profile_name: String,
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Pull { username, instance, profile_name }) => {
            let pw = get_password(Origin::Source);
            let user = User::new(username, instance);
            pull(user, pw, profile_name).await?;
            let instance_name = instance
                .host_str()
                .unwrap_or(instance.as_str());
            println!("Successfully pulled acount {}@{} to local profile '{}'.",
                    username, instance_name, profile_name);
        },
        None => {}
    }
    Ok(())
}
