use lemmy_api_common::lemmy_db_schema::{newtypes::CommunityId, source::community};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Community {
    pub name: String,
    pub id: CommunityId,
}

impl Community {
    pub fn new(community: &community::Community) -> Self {
        Community {
            name: community.name.to_owned(),
            id: community.id.clone(),
        }

    }
}
