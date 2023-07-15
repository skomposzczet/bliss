use crate::user::{User, Authorized, NotAuthorized};
use reqwest::{Client, Error};
use url::Url;
use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;

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
    pub async fn login(&self, user: User<NotAuthorized>, password: &str) -> Result<User<Authorized>, Error> {
        let url = api_path(&user.instance, "user/login");
        let params = person::Login {
            username_or_email: Sensitive::new(user.username.clone()),
            password: Sensitive::new(password.to_owned()),
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

}
