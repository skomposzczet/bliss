use lemmy_api_common::{lemmy_db_schema::{newtypes::DbUrl, source::community}, lemmy_db_views_actor::structs::CommunityView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Community {
    pub name: String,
    pub actor: DbUrl,
}

impl Community {
    pub fn new(community: &community::Community) -> Self {
        Community {
            name: community.name.to_owned(),
            actor: community.actor_id.clone(),
        }

    }

    pub fn is_same(&self, community: &CommunityView) -> bool {
        if self.name != community.community.name {
            return false;
        }
        if self.actor != community.community.actor_id {
            return false;
        }
        return true;
    }
}
