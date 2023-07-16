use std::{path::PathBuf, io::{Error, Write, ErrorKind}, fs::{create_dir_all, File, self}};

use super::Profile;
use home::home_dir;

const PROFILE_PATH_RELATIVE: &'static str = ".bliss/profiles/";
const PROFILE_FILENAME: &'static str = "profile.yml";

pub struct LocalProfile {
    name: String,
    profile: Profile,
}

impl LocalProfile {
    pub fn new(name: &str, profile: Profile) -> Self {
        LocalProfile {
            name: name.to_owned(),
            profile,
        }
    }

    fn path(profile_name: &str, filename: &str) -> Result<PathBuf, Error> {
        let mut path = home_dir()
            .ok_or(Error::new(ErrorKind::NotFound, "Impossible to get home directory."))?
            .join(PROFILE_PATH_RELATIVE)
            .join(&profile_name);
        create_dir_all(&path)?;
        path.push(filename);
        Ok(path)
    }

    pub fn load(profile_name: &str) -> Result<LocalProfile, Error> {
        let path = Self::path(&profile_name, PROFILE_FILENAME)?;
        let profile = fs::read_to_string(path)?;
        let profile = serde_yaml::from_str::<Profile>(&profile)
            .expect(&format!("Could not read current profile: {}.", profile_name));
        let lp = LocalProfile {
            name: profile_name.to_owned(),
            profile,
        };
        Ok(lp)
    }

    pub fn save(&mut self) -> Result<(), Error> {
        let path = Self::path(&self.name, PROFILE_FILENAME)?;
        let prev_profile = LocalProfile::load(&self.name);
        if prev_profile.is_ok() {
            self.profile.sync(prev_profile.unwrap().profile);
        }
        let profile = serde_yaml::to_string(&self.profile).unwrap(); 
        let mut file = File::create(path)?;
        file.write_all(profile.as_bytes())?;
        Ok(())
    }
}
