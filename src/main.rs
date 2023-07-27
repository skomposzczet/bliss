extern crate pretty_logger;
#[macro_use] extern crate log;
use log::LogLevelFilter;

mod user;
mod lemmy;
mod profile;
mod bliss;

use bliss::{Bliss, error::Error, util::{get_password, Origin}};
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

        #[arg(short, long, help="2FA token for source account")]
        token: Option<String>,

        #[arg(short, long, help="Local profile name")]
        profile_name: String,
    },
    /// Push account settings from local profile
    Push {
        #[arg(short, long, help="Destination username or email")]
        username: String,

        #[arg(short, long, help="Destination instance")]
        instance: Url,

        #[arg(short, long, help="2FA token for destination account")]
        token: Option<String>,

        #[arg(short, long, help="Local profile name")]
        profile_name: String,

        #[arg(short, long, help="Unfollows and unblocks communities and users if not followed or blocked in local profile")]
        subtractive: bool,

        #[arg(long, help="Parameters to ignore while pushing")]
        wno: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    pretty_logger::init_level(LogLevelFilter::Info).unwrap();
    let cli = Cli::parse();
    if let Err(err) = exec_command(&cli).await {
        error!("{}", err);
    }
}

async fn exec_command(cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        Some(Commands::Pull { username, instance, token, profile_name }) => {
            let pw = get_password(Origin::Source);
            let user = User::new(username, instance);
            let bliss = Bliss::new(user, pw, token.to_owned(), profile_name).await?;
            bliss.pull().await?;
        },
        Some(Commands::Push { username, instance, token, profile_name, subtractive, wno }) => {
            let pw = get_password(Origin::Destination);
            let user = User::new(username, instance);
            let bliss = Bliss::new(user, pw, token.to_owned(), profile_name).await?;
            bliss.push(*subtractive, wno).await?;
        },
        None => {}
    }
    Ok(()) 
}
