use url::Url;
use std::io::Write;

pub enum Origin {
    Source,
    Destination,
}

pub fn get_password(origin: Origin) -> String {
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

pub fn instance_host(instance: &Url) -> String {
    instance
        .host_str()
        .unwrap_or(instance.as_str())
        .to_string()
}

#[macro_export]
macro_rules! log_res {
    ( $e:expr ) => {
        match $e {
            Ok(_) => info!("Success"),
            Err(err) => warn!("Failed: {}", err),
        }
    }
}

