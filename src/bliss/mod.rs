pub mod error;
pub mod util;

use std::{time::Duration, thread, cell::Cell};
use lemmy_api_common::lemmy_db_schema::newtypes::{CommunityId, PersonId};
use crate::{api::Api, user::{User, Authorized, NotAuthorized}, profile::{Profile, local_profile::LocalProfile, community::Community, person::Person, Info}, bliss::util::instance_host, log_res};
use self::error::Error;

pub struct Bliss {
    user: User<Authorized>,
    api: Api,
    profile_name: String,
    subtractive: Cell<bool>,
}

impl Bliss {
    pub async fn new(user: User<NotAuthorized>, password: String, profile_name: &str) -> Result<Bliss, Error> {
        let api = Api::new();
        let user = api.login(user, password)
            .await
            .map_err(|err| Error::BlissError(format!("Failed while trying to login: {}", err)))?;
        let bliss = Bliss {
            user,
            api,
            profile_name: profile_name.to_owned(),
            subtractive: Cell::new(false),
        };
        Ok(bliss)
    }

    pub async fn pull(&self) -> Result<(), Error> {
        info!("Pulling {}@{} to local profile {}.",
                self.user.username, instance_host(&self.user.instance), self.profile_name);
        let site = self.api.site(&self.user)
            .await
            .map_err(|err| Error::ReqwestError(err))?;
        let profile = Profile::new(self.user.clone(), &site);
        let mut lp = LocalProfile::new(&self.profile_name, profile);
        lp.save().map_err(|err| Error::IoError(err))?;
        info!("Pulled successfully.");
        Ok(())
    }

    pub async fn push(&self, subtractive: bool, ignore: &[String]) -> Result<(), Error> {
        info!("Pushing {}@{} from local profile {}",
            self.user.username, instance_host(&self.user.instance), self.profile_name);
        self.subtractive.set(subtractive);
        let profile = LocalProfile::load(&self.profile_name)
            .map_err(|err| Error::IoError(err))?
            .profile
            .ignore_parameters(ignore);
        self.push_settings(profile.clone()).await?;
        self.push_info(&profile.info).await?;
        info!("Pushed successfully.");
        Ok(())
    }

    async fn push_settings(&self, profile: Profile) -> Result<(), Error> {
        info!("Uploading settings...");
        self.api.save_user_settings(&self.user, profile)
            .await
            .map_err(|err| Error::ReqwestError(err))?;
        info!("Successfully uploaded settings.");
        Ok(())
    }

    async fn push_info(&self, info: &Info) -> Result<(), Error> {
        let site = self.api.site(&self.user).await
            .map_err(|err| Error::ReqwestError(err))?;
        let dst_profile = Profile::new(self.user.clone(), &site);
        let dst_info = dst_profile.info;
        let rate_limit = site
            .site_view
            .local_site_rate_limit
            .message_per_second;
        let sleep_time = Duration::from_millis((1000 as f64 / rate_limit as f64).ceil() as u64);
        self.push_communities(info, &dst_info, sleep_time).await;
        self.push_users(info, &dst_info, sleep_time).await;
        if self.subtractive.get() {
            let undo_info = dst_info.subtract(info);
            self.subtractive_push_info(&undo_info, sleep_time).await;
        }
        Ok(())
    }

    async fn push_communities(&self, info: &Info, dst_info: &Info, sleep_time: Duration) {
        let iterator = info
            .communities_follows
            .iter()
            .filter(|c| !dst_info.communities_follows.contains(c));
        for community in iterator {
            log_res!(self.follow_community(&community).await);
            thread::sleep(sleep_time);
        }
        let iterator = info
            .communities_blocks
            .iter()
            .filter(|c| !dst_info.communities_blocks.contains(c));
        for community in iterator {
            log_res!(self.block_community(&community).await);
            thread::sleep(sleep_time);
        }
    }

    async fn push_users(&self, info: &Info, dst_info: &Info, sleep_time: Duration) {
        let iterator = info
            .people_blocks
            .iter()
            .filter(|p| !dst_info.people_blocks.contains(p));
        for person in iterator {
            log_res!(self.block_person(&person).await);
            thread::sleep(sleep_time);
        }
    }

    async fn subtractive_push_info(&self, undo_info: &Info, sleep_time: Duration) {
        for community in undo_info.communities_follows.iter() {
            log_res!(self.unfollow_community(&community).await);
            thread::sleep(sleep_time);
        }
        for community in undo_info.communities_blocks.iter() {
            log_res!(self.unblock_community(&community).await);
            thread::sleep(sleep_time);
        }
        for user in undo_info.people_blocks.iter() {
            log_res!(self.unblock_person(&user).await);
            thread::sleep(sleep_time);
        }
    }

    async fn follow_community(&self, community: &Community) -> Result<(), Error> {
        info!("Following {}...", community.name);
        let community_id = self.find_community(&community)
            .await?;
        self.api.follow_community(&self.user, &community_id, true)
            .await?;
        Ok(())
    }

    async fn unfollow_community(&self, community: &Community) -> Result<(), Error> {
        info!("Unfollowing {}...", community.name);
        self.api.follow_community(&self.user, &community.id, false)
            .await?;
        Ok(())
    }

    async fn block_community(&self, community: &Community) -> Result<(), Error> {
        info!("Blocking {}...", community.name);
        let community_id = self.find_community(community)
            .await?;
        self.api.block_community(&self.user, &community_id, true)
            .await?;
        Ok(())
    }

    async fn unblock_community(&self, community: &Community) -> Result<(), Error> {
        info!("Unblocking {}...", community.name);
        self.api.block_community(&self.user, &community.id, false)
            .await?;
        Ok(())
    }

    async fn block_person(&self, person: &Person) -> Result<(), Error> {
        info!("Blocking {}...", person.username);
        let person_id = self.find_person(person)
            .await?;
        self.api.block_person(&self.user, &person_id, true)
            .await?;
        Ok(())
    }

    async fn unblock_person(&self, person: &Person) -> Result<(), Error> {
        info!("Unblocking {}...", person.username);
        let person_id = self.find_person(person)
            .await?;
        self.api.block_person(&self.user, &person_id, false)
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
