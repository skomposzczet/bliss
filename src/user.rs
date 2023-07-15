use url::Url;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct NotAuthorized;
#[derive(Clone)]
pub struct Authorized;

#[derive(Clone)]
pub struct User<AuthState = NotAuthorized> {
    pub username: String,
    pub instance: Url,
    jwtoken: Option<String>,
    state: PhantomData<AuthState>,
}

impl User<NotAuthorized> {
    pub fn authorize(self, jwt: String) -> User<Authorized> {
        User {
            username: self.username,
            instance: self.instance,
            jwtoken: Some(jwt),
            state: PhantomData,
        }
    }
}

impl User<Authorized> {
    pub fn token(&self) -> &str {
        self.jwtoken.as_ref().unwrap()
    }
}

impl User {
    pub fn new(username: &str, url: Url) -> Self {
        User {
            username: username.into(),
            instance: url,
            jwtoken: None,
            state: PhantomData,
        }
    }
}
