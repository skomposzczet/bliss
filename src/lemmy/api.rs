use crate::profile::Profile;
use crate::profile::community::Community;
use crate::profile::person::Person;
use crate::user::{User, Authorized, NotAuthorized};
use lemmy_api_common::community::{CommunityResponse, FollowCommunity, BlockCommunity, BlockCommunityResponse};
use lemmy_api_common::lemmy_db_schema::{SearchType, SortType};
use lemmy_api_common::lemmy_db_schema::newtypes::{CommunityId, PersonId, DbUrl};
use reqwest::multipart::{Part, Form};
use reqwest::Client;
use url::Url;
use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person::{self, BlockPersonResponse, BlockPerson};
use lemmy_api_common::site;

use super::LemmyError;
use super::image::UploadImageResponse;

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
    pub async fn login(&self, user: User<NotAuthorized>, password: String, token: Option<String>) -> Result<User<Authorized>, LemmyError> {
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

    pub async fn site(&self, user: &User<Authorized>) -> Result<site::GetSiteResponse, LemmyError> {
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

    pub async fn save_user_settings(&self, user: &User<Authorized>, profile: Profile) -> Result<person::LoginResponse, LemmyError> {
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

    pub async fn search_community(&self, user: &User<Authorized>, community: &Community) -> Result<site::SearchResponse, LemmyError> {
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

    pub async fn search_person(&self, user: &User<Authorized>, person: &Person) -> Result<site::SearchResponse, LemmyError> {
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


    pub async fn follow_community(&self, user: &User<Authorized>, id: &CommunityId, follow: bool) -> Result<CommunityResponse, LemmyError> {
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

    pub async fn block_community(&self, user: &User<Authorized>, id: &CommunityId, block: bool) -> Result<BlockCommunityResponse, LemmyError> {
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

    pub async fn block_person(&self, user: &User<Authorized>, id: &PersonId, block: bool) -> Result<BlockPersonResponse, LemmyError> {
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

    pub async fn download_image(&self, url: &Option<DbUrl>) -> Result<Option<bytes::Bytes>, LemmyError> {
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

    pub async fn upload_image(&self, user: &User<Authorized>, bytes: Vec<u8>) -> Result<Option<Url>, LemmyError> {
        let path = user.instance
            .join("pictrs/image")
            .unwrap();
        let part = Part::bytes(bytes)
            .file_name("image")
            .mime_str("image/png")
            .unwrap();
        let form = Form::new()
            .part("images[]", part);
        let response = self.client
            .post(path)
            .header("cookie", format!("jwt={}", user.token()))
            .multipart(form)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(LemmyError::ResponeError(format!("Status is {}", response.status())));
        }
        let res: UploadImageResponse = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        if res.msg != "ok" {
            return Err(LemmyError::ResponeError(format!("Msg is {}", res.msg)));
        }
        let url = user.instance
            .join(&format!("pictrs/image/{}", res.files[0].file))
            .unwrap();
        debug!("URL: {}", url);
        Ok(Some(url))
    }
}
