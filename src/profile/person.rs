use lemmy_api_common::{lemmy_db_schema::{newtypes::DbUrl, source::person}, lemmy_db_views_actor::structs::PersonView};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Person {
    pub username: String,
    pub actor: DbUrl,
}

impl Person {
    pub fn new(person: &person::Person) -> Self {
        Person {
            username: person.name.clone(),
            actor: person.actor_id.clone(),
        }
    }

    pub fn is_same(&self, person: &PersonView) -> bool {
        if self.username != person.person.name {
            return false;
        }
        if self.actor != person.person.actor_id {
            return false;
        }
        return true;
    }
}
