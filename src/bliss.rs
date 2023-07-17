use std::{time::Duration, thread};

use lemmy_api_common::lemmy_db_schema::newtypes::CommunityId;

use crate::{api::Api, user::{User, Authorized}, profile::{Profile, local_profile::LocalProfile, community::Community, person::Person}};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError( #[from] reqwest::Error ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
    #[error("Error: {0}")]
    BlissError(String),
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
        let result = follow_community(&api, &user, &community).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    for community in info.communities_blocks.iter() {
        let result = block_community(&api, &user, &community).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    for person in info.people_blocks.iter() {
        let result = block_person(&api, &user, &person).await;
        log_result(result);
        thread::sleep(sleep_time);
    }
    Ok(())
}

fn log_result<T>(result: Result<T, Error>) {
    match result {
        Ok(_) => println!("Success"),
        Err(err) => println!("Failed: {}", err),
    }
}

async fn find_community(api: &Api, user: &User<Authorized>, community: &Community) -> Result<CommunityId, Error> {
    let response = api.search_community(&user, &community).await
        .map_err(|err| Error::ReqwestError(err))?;
    let community_id: Option<CommunityId> = {
        let found: Vec<_> = response.communities
            .iter()
            .filter_map(|comm| if community.is_same(comm) { Some(comm.community.id) } else { None })
            .collect();
        if found.len() == 1 {
            Some(found[0])
        } else {
            None
        }
    };
    let community_id = community_id
        .ok_or(Error::BlissError(format!("Unable to find community: {}", community.actor)));
    community_id
}

async fn follow_community(api: &Api, user: &User<Authorized>, community: &Community) -> Result<(), Error> {
    println!("Following {}...", community.name);
    let community_id = find_community(&api, &user, &community)
        .await?;
    api.follow_community(&user, &community_id)
        .await?;
    Ok(())
}

async fn block_community(api: &Api, user: &User<Authorized>, community: &Community) -> Result<(), Error> {
    println!("Blocking {}...", community.name);
    let community_id = find_community(&api, &user, &community)
        .await?;
    api.block_community(&user, &community_id)
        .await?;
    Ok(())
}

async fn block_person(api: &Api, user: &User<Authorized>, person: &Person) -> Result<(), Error> {
    println!("Blocking {}...", person.username);
    api.block_person(&user, &person.id)
        .await?;
    Ok(())
}
