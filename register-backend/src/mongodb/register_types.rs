use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize_repr, Serialize_repr, FromPrimitive)]
#[repr(i32)]
pub enum RecordState {
    Unspecified = 0,
    Draft = 1,
    Created = 2,
    CollectClientInside = 3,
    CollectClientSignature = 4,
    CollectClientOutside = 5,
    CollectPqrsSignature = 6,
    ReturnClientInside = 7,
    ReturnClientSignature = 8,
    ReturnClientOutside = 9,
    ReturnPqrsSignature = 10,
    Completed = 11,
}

impl From<RecordState> for Bson {
    fn from(value: RecordState) -> Self {
        Self::Int32(value as i32)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Signer {
    pub name: String,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trace {
    pub inside: Option<i64>,
    pub outside: Option<i64>,
    pub client: Option<Signer>,
    pub pqrs: Option<Signer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Traces {
    pub collected: Option<Trace>,
    pub returned: Option<Trace>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub api_version: i32,
    pub created: Option<i64>,
    pub summary: String,
    pub traces: Option<Traces>,
    pub state: RecordState,
}