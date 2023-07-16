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
