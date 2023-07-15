use lemmy_api_common::{lemmy_db_schema::newtypes::CommunityId, lemmy_db_views_actor::structs::CommunityFollowerView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Community {
    pub name: String,
    pub id: CommunityId,
}

impl Community {
    pub fn new(community_view: &CommunityFollowerView) -> Self {
        Community {
            name: community_view.community.name.to_owned(),
            id: community_view.community.id.clone(),
        }

    }
}
