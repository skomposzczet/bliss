use std::{path::{PathBuf, Path}, io::{Error, Write, ErrorKind, Read}, fs::{create_dir_all, File, self}};
use super::Profile;
use bytes::Bytes;
use home::home_dir;


const PROFILE_PATH_RELATIVE: &'static str = ".bliss/profiles/";
const PROFILE_FILENAME: &'static str = "profile.yml";
const AVATAR_FILENAME: &'static str = "avatar.png";
const BANNER_FILENAME: &'static str = "banner.png";

pub struct LocalProfile {
    pub name: String,
    pub profile: Profile,
}

impl LocalProfile {
    pub fn new(name: &str, profile: Profile) -> Self {
        LocalProfile {
            name: name.to_owned(),
            profile,
        }
    }

    pub fn save_avatar(&self, avatar: Option<Bytes>) -> Result<bool, Error> {
        self.save_image(avatar, AVATAR_FILENAME, "avatar")
    }

    pub fn save_banner(&self, banner: Option<Bytes>) -> Result<bool, Error> {
        self.save_image(banner, BANNER_FILENAME, "banner")
    }

    fn save_image(&self, image: Option<Bytes>, filename: &str, debug_name: &str) -> Result<bool, Error> {
        if image.is_none() {
            return Ok(false);
        }
        let image = image::load_from_memory(&image.unwrap())
            .map_err(|err| Error::new(ErrorKind::Other, format!("Failed to load image {}; : {}.", debug_name, err)))?;
        let path = Self::path(&self.name, filename)?;
        let path = Path::new(&path);
        image.save(&path)
            .map_err(|err| Error::new(ErrorKind::Other, format!("Failed to save image; {}", err)))?;
        Ok(true)
    }

    pub fn load_avatar(&self) -> Result<Option<Vec<u8>>, Error> {
        self.load_image(AVATAR_FILENAME)
    }

    pub fn load_banner(&self) -> Result<Option<Vec<u8>>, Error> {
        self.load_image(BANNER_FILENAME)
    }

    fn load_image(&self, filename: &str) -> Result<Option<Vec<u8>>, Error> {
        let path = Self::path(&self.name, filename)?;
        if !path.exists() {
            return Ok(None);
        }
        let mut file = File::open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(Some(bytes))
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
