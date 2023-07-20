use std::{time::Duration, thread};

use lemmy_api_common::lemmy_db_schema::newtypes::{CommunityId, PersonId};

use crate::{api::Api, user::{User, Authorized, NotAuthorized}, profile::{Profile, local_profile::LocalProfile, community::Community, person::Person}};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError( #[from] reqwest::Error ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
    #[error("Error: {0}")]
    BlissError(String),
}

fn log_result<T>(result: Result<T, Error>) {
    match result {
        Ok(_) => info!("Success"),
        Err(err) => error!("Failed: {}", err),
    }
}

pub struct Bliss {
    user: User<Authorized>,
    api: Api,
    profile_name: String,
}

impl Bliss {
    pub async fn new(user: User<NotAuthorized>, password: String, profile_name: &str) -> Result<Bliss, Error> {
        let api = Api::new();
        let user = api.login(user, password)
            .await
            .map_err(|err| Error::ReqwestError(err))?;
        let bliss = Bliss {
            user,
            api,
            profile_name: profile_name.to_owned(),
        };
        Ok(bliss)
    }

    pub async fn pull(&self) -> Result<(), Error> {
        let site = self.api.site(&self.user)
            .await
            .map_err(|err| Error::ReqwestError(err))?;
        let profile = Profile::new(self.user.clone(), &site);
        let mut lp = LocalProfile::new(&self.profile_name, profile);
        lp.save().map_err(|err| Error::IoError(err))?;
        Ok(())
    }

    pub async fn push(&self) -> Result<(), Error> {
        let profile = LocalProfile::load(&self.profile_name)
            .map_err(|err| Error::IoError(err))?
            .profile;
        let info = profile.info.clone();
        info!("Uploading settings...");
        self.api.save_user_settings(&self.user, profile)
            .await
            .map_err(|err| Error::ReqwestError(err))?;
        info!("Successfully uploaded settings.");

        let rate_limit = self.api.site(&self.user).await
            .map_err(|err| Error::ReqwestError(err))?
            .site_view
            .local_site_rate_limit
            .message_per_second;
        let sleep_time = Duration::from_millis((1000 as f64 / rate_limit as f64).ceil() as u64);
        for community in info.communities_follows.iter() {
            let result = self.follow_community(&community).await;
            log_result(result);
            thread::sleep(sleep_time);
        }
        for community in info.communities_blocks.iter() {
            let result = self.block_community(&community).await;
            log_result(result);
            thread::sleep(sleep_time);
        }
        for person in info.people_blocks.iter() {
            let result = self.block_person(&person).await;
            log_result(result);
            thread::sleep(sleep_time);
        }
        Ok(())
    }

    async fn follow_community(&self, community: &Community) -> Result<(), Error> {
        info!("Following {}...", community.name);
        let community_id = self.find_community(&community)
            .await?;
        self.api.follow_community(&self.user, &community_id)
            .await?;
        Ok(())
    }

    async fn block_community(&self, community: &Community) -> Result<(), Error> {
        info!("Blocking {}...", community.name);
        let community_id = self.find_community(community)
            .await?;
        self.api.block_community(&self.user, &community_id)
            .await?;
        Ok(())
    }

    async fn block_person(&self, person: &Person) -> Result<(), Error> {
        info!("Blocking {}...", person.username);
        let person_id = self.find_person(person)
            .await?;
        self.api.block_person(&self.user, &person_id)
            .await?;
        Ok(())
    }

    async fn find_community(&self, community: &Community) -> Result<CommunityId, Error> {
        let response = self.api.search_community(&self.user, &community).await
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

    async fn find_person(&self, person: &Person) -> Result<PersonId, Error> {
        let response = self.api.search_person(&self.user, &person).await
            .map_err(|err| Error::ReqwestError(err))?;
        let person_id: Option<PersonId> = {
            let found: Vec<_> = response.users
                .iter()
                .filter_map(|pers| if person.is_same(pers) { Some(pers.person.id) } else { None })
                .collect();
            if found.len() == 1 {
                Some(found[0])
            } else {
                None
            }
        };
        let person_id = person_id
            .ok_or(Error::BlissError(format!("Unable to find user: {}", person.actor)));
        person_id
    }
}


