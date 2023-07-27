use crate::profile::Profile;
use crate::profile::community::Community;
use crate::profile::person::Person;
use crate::user::{User, Authorized, NotAuthorized};
use lemmy_api_common::community::{CommunityResponse, FollowCommunity, BlockCommunity, BlockCommunityResponse};
use lemmy_api_common::lemmy_db_schema::{SearchType, SortType};
use lemmy_api_common::lemmy_db_schema::newtypes::{CommunityId, PersonId, DbUrl};
use reqwest::{Client, Error};
use url::Url;
use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person::{self, BlockPersonResponse, BlockPerson};
use lemmy_api_common::site;

const API_BASE: &'static str = "/api/v3"; 

fn api_path(instance: &Url, path: &str) -> Url {
    let path = format!("{}/{}", API_BASE, path);
    instance.join(&path).unwrap()
}

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        Api{
            client: Client::new(),
        }
    }
    pub async fn login(&self, user: User<NotAuthorized>, password: String, token: Option<String>) -> Result<User<Authorized>, Error> {
        let url = api_path(&user.instance, "user/login");
        let params = person::Login {
            username_or_email: Sensitive::new(user.username.clone()),
            password: Sensitive::new(password),
            totp_2fa_token: token,
        };
        let response = self.client
            .post(url)
            .json(&params)
            .send()
            .await;
        let jwt = response?
            .json::<person::LoginResponse>().await?
            .jwt.unwrap()
            .clone();
        
        Ok(user.authorize(jwt.to_string()))
    }

    pub async fn site(&self, user: &User<Authorized>) -> Result<site::GetSiteResponse, Error> {
        let url = api_path(&user.instance, "site");
        let params = site::GetSite {
            auth: Some(Sensitive::from(user.token()))
        };
        let response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;
        let result = response.json::<site::GetSiteResponse>().await.unwrap();
        Ok(result)
    }

    pub async fn save_user_settings(&self, user: &User<Authorized>, profile: Profile) -> Result<person::LoginResponse, Error> {
        let url = api_path(&user.instance, "user/save_user_settings");
        let mut settings = person::SaveUserSettings::from(profile);
        settings.auth = Sensitive::from(user.token());
        let response = self.client
            .put(url)
            .json(&settings)
            .send()
            .await?;
        let result = response.json::<person::LoginResponse>().await?;
        Ok(result)
    }

    pub async fn search_community(&self, user: &User<Authorized>, community: &Community) -> Result<site::SearchResponse, Error> {
        let url = api_path(&user.instance, "search");
        let params = site::Search {
            q: community.name.clone(),
            type_: Some(SearchType::Communities),
            sort: Some(SortType::TopAll),
            auth: Some(Sensitive::from(user.token())),
            ..Default::default()
        };
        let response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;
        let result = response.json::<site::SearchResponse>().await.unwrap();
        Ok(result)
    }

    pub async fn search_person(&self, user: &User<Authorized>, person: &Person) -> Result<site::SearchResponse, Error> {
        let url = api_path(&user.instance, "search");
        let params = site::Search {
            q: person.username.clone(),
            type_: Some(SearchType::Users),
            sort: Some(SortType::TopAll),
            auth: Some(Sensitive::from(user.token())),
            ..Default::default()
        };
        let response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;
        let result = response.json::<site::SearchResponse>().await.unwrap();
        Ok(result)
    }


    pub async fn follow_community(&self, user: &User<Authorized>, id: &CommunityId, follow: bool) -> Result<CommunityResponse, Error> {
        let url = api_path(&user.instance, "community/follow");
        let params = FollowCommunity {
            community_id: id.clone(),
            follow,
            auth: Sensitive::from(user.token()),
        };
        let response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;
        let result = response.json::<CommunityResponse>().await?;
        Ok(result)
    }

    pub async fn block_community(&self, user: &User<Authorized>, id: &CommunityId, block: bool) -> Result<BlockCommunityResponse, Error> {
        let url = api_path(&user.instance, "community/block");
        let params = BlockCommunity {
            community_id: id.clone(),
            block,
            auth: Sensitive::from(user.token()),
        };
        let response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;
        let result = response.json::<BlockCommunityResponse>().await?;
        Ok(result)
    }

    pub async fn block_person(&self, user: &User<Authorized>, id: &PersonId, block: bool) -> Result<BlockPersonResponse, Error> {
        let url = api_path(&user.instance, "user/block");
        let params = BlockPerson {
            person_id: id.clone(),
            block,
            auth: Sensitive::from(user.token()),
        };
        let response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;
        let result = response.json::<BlockPersonResponse>().await?;
        Ok(result)
    }

    pub async fn get_image(&self, url: &Option<DbUrl>) -> Result<Option<bytes::Bytes>, Error> {
        if url.is_none() {
            return Ok(None);
        }
        let url = url
            .as_ref()
            .unwrap()
            .inner()
            .to_owned();
        let bytes = self.client
            .get(url)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(Some(bytes))
    }
}
