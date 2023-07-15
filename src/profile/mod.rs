use chrono::{DateTime, Local};
use lemmy_api_common::site::GetSiteResponse;
use serde::{Serialize, Deserialize};

use crate::user::User;

use self::community::Community;

pub mod community;

#[derive(Serialize, Deserialize)]
struct Meta {
    username: String,
    instance: url::Url,
    date_created: DateTime<Local>,
    date_updated: DateTime<Local>,
}

impl<T> From<User<T>> for Meta {
    fn from(user: User<T>) -> Self {
        let date = Local::now();
        Meta {
            username: user.username.clone(),
            instance: user.instance.clone(),
            date_created: date,
            date_updated: date,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Info {
    communities_blocks: Vec<Community>,
    communities_follows: Vec<Community>,
}

impl Info {
    fn new(site: &GetSiteResponse) -> Self {
        let com_block: Vec<Community> = site.my_user.clone().unwrap().community_blocks
            .iter()
            .map(|community| Community::new(&community.community))
            .collect();
        let com_follow: Vec<Community> = site.my_user.clone().unwrap().follows
            .iter()
            .map(|community| Community::new(&community.community))
            .collect();

        Info {
            communities_blocks: com_block,
            communities_follows: com_follow,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    meta: Meta,
    info: Info,
}

impl Profile {
    pub fn new<T>(user: User<T>, site: &GetSiteResponse) -> Self {
        Profile {
            meta: Meta::from(user),
            info: Info::new(site),
        }
    }
}
