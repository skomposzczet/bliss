use chrono::{DateTime, Local};
use lemmy_api_common::{site::GetSiteResponse, lemmy_db_schema::{SortType, ListingType, newtypes::LanguageId}, person::SaveUserSettings, sensitive::Sensitive};
use serde::{Serialize, Deserialize};

use crate::user::User;

use self::{community::Community, person::Person};

pub mod community;
pub mod person;
pub mod local_profile;

#[derive(Serialize, Deserialize, Clone)]
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

impl Meta {
    fn touch(&mut self) {
        self.date_updated = Local::now();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Info {
    pub communities_blocks: Vec<Community>,
    pub communities_follows: Vec<Community>,
    pub people_blocks: Vec<Person>,
    pub bio: Option<String>,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
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
        let ppl_block: Vec<Person> = site.my_user.clone().unwrap().person_blocks
            .iter()
            .map(|person| Person::new(&person.target))
            .collect();

        Info {
            communities_blocks: com_block,
            communities_follows: com_follow,
            people_blocks: ppl_block,
            bio: site.my_user.clone().unwrap().local_user_view.person.bio,
            display_name: site.my_user.clone().unwrap().local_user_view.person.display_name,
            avatar: None,
            banner: None,
        }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let com_block = self.communities_blocks
            .clone()
            .into_iter()
            .filter(|c| !other.communities_blocks.contains(c))
            .collect();
        let com_follow = self.communities_follows
            .clone()
            .into_iter()
            .filter(|c| !other.communities_follows.contains(c))
            .collect();
        let ppl_block = self.people_blocks
            .clone()
            .into_iter()
            .filter(|p| !other.people_blocks.contains(p))
            .collect();
        Info {
            communities_blocks: com_block,
            communities_follows: com_follow,
            people_blocks: ppl_block,
            bio: self.bio.clone(),
            display_name: self.display_name.clone(),
            avatar: None,
            banner: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    default_sort_type: SortType,
    default_listing_type: ListingType,
    theme: String,
    interface_language: String,
    email: Option<String>,
    matrix_user_id: Option<String>,
    show_nsfw: bool,
    show_scores: bool,
    show_avatars: bool,
    show_bot_accounts: bool,
    show_read_posts: bool,
    show_new_post_notifs: bool,
    open_links_in_new_tab: bool,
    send_notifications_to_email: bool,
    bot_account: bool,
    discussion_languages: Vec<LanguageId>,
}

impl Settings {
    fn new(site: &GetSiteResponse) -> Settings {
        let local_user = site.my_user
            .clone()
            .unwrap()
            .local_user_view
            .local_user;
        let user = site.my_user
            .clone()
            .unwrap()
            .local_user_view
            .person;

        Settings {
            default_sort_type: local_user.default_sort_type,
            default_listing_type: local_user.default_listing_type,
            theme: local_user.theme,
            interface_language: local_user.interface_language,
            email: local_user.email,
            matrix_user_id: user.matrix_user_id,
            show_nsfw: local_user.show_nsfw,
            show_scores: local_user.show_scores,
            show_avatars: local_user.show_avatars,
            show_bot_accounts: local_user.show_bot_accounts,
            show_read_posts: local_user.show_read_posts,
            show_new_post_notifs: local_user.show_new_post_notifs,
            open_links_in_new_tab: local_user.open_links_in_new_tab,
            send_notifications_to_email: local_user.send_notifications_to_email,
            bot_account: user.bot_account,
            discussion_languages: site.discussion_languages.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    meta: Meta,
    pub info: Info,
    settings: Settings,
}

impl Profile {
    pub fn new<T>(user: User<T>, site: &GetSiteResponse) -> Self {
        Profile {
            meta: Meta::from(user),
            info: Info::new(site),
            settings: Settings::new(site),
        }
    }

    pub fn sync(&mut self, other: Self) {
        self.meta = other.meta;
        self.meta.touch();
    }

    pub fn ignore_parameters(mut self, ignore: &[String]) -> Self {
        for param_name in ignore.iter() {
            info!("Ignoring parameter \"{}\"...", param_name);
            match param_name.as_str() {
                "email" => self.settings.email = None,
                "matrix_user_id" => self.settings.matrix_user_id = None,
                "bio" => self.info.bio = None,
                "display_name" => self.info.display_name = None,
                "avatar" => self.info.avatar = None,
                "banner" => self.info.banner = None,
                _ => error!("Unknown parameter \"{}\". Skipping.", param_name),
            }
        }
        self
    }

}

impl From<Profile> for SaveUserSettings {
    fn from(profile: Profile) -> Self {
        SaveUserSettings {
            default_sort_type: Some(profile.settings.default_sort_type),
            default_listing_type: Some(profile.settings.default_listing_type),
            theme: Some(profile.settings.theme),
            interface_language: Some(profile.settings.interface_language),
            email: match profile.settings.email {
                Some(email) => Some(Sensitive::new(email)),
                None => None,
            },
            matrix_user_id: profile.settings.matrix_user_id,
            show_nsfw: Some(profile.settings.show_nsfw),
            show_scores: Some(profile.settings.show_scores),
            show_avatars: Some(profile.settings.show_avatars),
            show_bot_accounts: Some(profile.settings.show_bot_accounts),
            show_read_posts: Some(profile.settings.show_read_posts),
            show_new_post_notifs: Some(profile.settings.show_new_post_notifs),
            open_links_in_new_tab: Some(profile.settings.open_links_in_new_tab),
            send_notifications_to_email: Some(profile.settings.send_notifications_to_email),
            bot_account: Some(profile.settings.bot_account),
            discussion_languages: Some(profile.settings.discussion_languages),
            bio: profile.info.bio,
            display_name: profile.info.display_name,
            avatar: profile.info.avatar,
            banner: profile.info.banner,
            generate_totp_2fa: None,
            auth: Sensitive::from(""),
        }
    }
}
