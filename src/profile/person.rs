use lemmy_api_common::lemmy_db_schema::{newtypes::PersonId, source::person};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Person {
    pub username: String,
    pub id: PersonId,
}

impl Person {
    pub fn new(person: &person::Person) -> Self {
        Person {
            username: person.name.clone(),
            id: person.id,
        }
    }
}
