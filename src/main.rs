extern crate pretty_logger;
#[macro_use] extern crate log;
use log::LogLevelFilter;

mod user;
mod api;
mod profile;
mod bliss;

use std::io::Write;
use bliss::{pull, Error};
use clap::{Parser, Subcommand};
use url::Url;
use user::User;

use crate::bliss::push;

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
    /// Push account settings from local profile
    Push {
        #[arg(short, long, help="Destination username or email")]
        username: String,

        #[arg(short, long, help="Destination instance")]
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

fn instance_host(instance: &Url) -> String {
    instance
        .host_str()
        .unwrap_or(instance.as_str())
        .to_string()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_logger::init_level(LogLevelFilter::Info).unwrap();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Pull { username, instance, profile_name }) => {
            let pw = get_password(Origin::Source);
            let user = User::new(username, instance);
            pull(user, pw, profile_name).await?;
            info!("Successfully pulled account {}@{} to local profile '{}'.",
                    username, instance_host(&instance), profile_name);
        },
        Some(Commands::Push { username, instance, profile_name }) => {
            let pw = get_password(Origin::Destination);
            let user = User::new(username, instance);
            push(user, pw, profile_name).await?;
            info!("Successfully pushed to account {}@{} from local profile '{}'.",
                    username, instance_host(&instance), profile_name);
        }
        None => {}
    }
    Ok(())
}
