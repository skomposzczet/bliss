use crate::profile::Profile;
use crate::user::{User, Authorized, NotAuthorized};
use reqwest::{Client, Error};
use url::Url;
use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;
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
    pub async fn login(&self, user: User<NotAuthorized>, password: String) -> Result<User<Authorized>, Error> {
        let url = api_path(&user.instance, "user/login");
        let params = person::Login {
            username_or_email: Sensitive::new(user.username.clone()),
            password: Sensitive::new(password),
            ..Default::default()
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
            auth: Some(Sensitive::new(user.token().to_owned()))
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
        settings.auth = Sensitive::new(user.token().to_owned());
        let response = self.client
            .put(url)
            .json(&settings)
            .send()
            .await?;
        let result = response.json::<person::LoginResponse>().await?;
        Ok(result)
    }

}
