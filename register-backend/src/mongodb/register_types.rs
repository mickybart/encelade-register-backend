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
pub(crate) struct Signer {
    pub(crate) name: String,
    pub(crate) signature: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Trace {
    pub(crate) inside: Option<i64>,
    pub(crate) outside: Option<i64>,
    pub(crate) client: Option<Signer>,
    pub(crate) pqrs: Option<Signer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Traces {
    pub(crate) collected: Option<Trace>,
    pub(crate) returned: Option<Trace>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Record {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) api_version: i32,
    pub(crate) created: Option<i64>,
    pub(crate) summary: String,
    pub(crate) traces: Option<Traces>,
    pub(crate) state: RecordState,
}