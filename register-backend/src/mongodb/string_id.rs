use core::fmt;

use mongodb::{bson::oid::ObjectId, error::Error};

pub struct StringId(pub String);

impl StringId {
    pub fn to_object_id(&self) -> Result<ObjectId, Error> {
        ObjectId::parse_str(&self.0).map_err(|e| Error::custom(e))
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}