use std::{time::Duration, thread};

use crate::{api::Api, user::User, profile::{Profile, local_profile::LocalProfile}};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError( #[from] reqwest::Error ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
}

pub async fn pull(user: User, password: String, profile_name: &str) -> Result<(), Error> {
    let api = Api::new();
    let user = api.login(user, password)
        .await
        .map_err(|err| Error::ReqwestError(err))?;
    let site = api.site(&user)
        .await
        .map_err(|err| Error::ReqwestError(err))?;
    let profile = Profile::new(user, &site);
    let mut lp = LocalProfile::new(profile_name, profile);
    lp.save().map_err(|err| Error::IoError(err))?;
    Ok(())
}

pub async fn push(user: User, password: String, profile_name: &str) -> Result<(), Error> {
    let api = Api::new();
    let user = api.login(user, password)
        .await
        .map_err(|err| Error::ReqwestError(err))?;
    let profile = LocalProfile::load(&profile_name)
        .map_err(|err| Error::IoError(err))?
        .profile;
    let info = profile.info.clone();
    println!("Uploading settings...");
    api.save_user_settings(&user, profile)
        .await
        .map_err(|err| Error::ReqwestError(err))?;
    println!("Successfully uploaded settings.");

    let rate_limit = api.site(&user).await
        .map_err(|err| Error::ReqwestError(err))?
        .site_view
        .local_site_rate_limit
        .message_per_second;
    let sleep_time = Duration::from_millis((1000 as f64 / rate_limit as f64).ceil() as u64);
    for community in info.communities_follows.iter() {
        println!("Following {}...", community.name);
        let result = api.follow_community(&user, &community.id).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    for community in info.communities_blocks.iter() {
        println!("Blocking {}...", community.name);
        let result = api.block_community(&user, &community.id).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    for person in info.people_blocks.iter() {
        println!("Blocking {}...", person.username);
        let result = api.block_person(&user, &person.id).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    Ok(())
}

fn log_result<T>(result: Result<T, reqwest::Error>) {
    match result {
        Ok(_) => println!("Success"),
        Err(err) => println!("Failed: {}", err),
    }
}
