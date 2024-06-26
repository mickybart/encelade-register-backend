//! MongoDB abstraction for Register collection

mod register_types;
mod string_id;
mod traces_for;

use crate::config::MongoDbConfig;

pub(crate) use self::register_types::{Record, RecordState, Signer, Trace};
pub(crate) use self::string_id::StringId;
pub(crate) use self::traces_for::{SignatureTraceFor, TimeTraceFor};

use std::time::Duration;

use chrono::Utc;
use mongodb::change_stream::event::ChangeStreamEvent;
use mongodb::change_stream::ChangeStream;
use mongodb::options::{ChangeStreamOptions, FullDocumentType};
use mongodb::Cursor;
use mongodb::{
    bson::{doc, to_bson},
    error::Error,
    options::ClientOptions,
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Collection,
};

/// A MongoDB Collection of [Record] type
pub(crate) struct Mongo {
    pub(crate) register: Collection<Record>,
}

static API_VERSION_1: i32 = 1;

impl Mongo {
    pub(crate) async fn new(config: MongoDbConfig) -> Result<Self, Error> {
        let client_options = ClientOptions::parse(&config.uri).await?;

        let client = Client::with_options(client_options)?;

        let register = client.database(&config.db).collection(&config.collection);

        Ok(Mongo { register })
    }

    pub(crate) async fn insert_draft(&self, summary: String) -> Result<InsertOneResult, Error> {
        let draft = Record {
            id: None,
            api_version: API_VERSION_1,
            created: Some(Utc::now().timestamp()),
            summary: summary,
            traces: None,
            state: RecordState::Draft,
        };

        self.register.insert_one(draft, None).await
    }

    pub(crate) async fn update_draft(
        &self,
        id: StringId,
        summary: String,
    ) -> Result<UpdateResult, Error> {
        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };
        let update = doc! {
            "$set": {
                "summary": summary,
            },
        };

        self.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub(crate) async fn delete_draft(&self, id: StringId) -> Result<DeleteResult, Error> {
        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };

        self.register.delete_one(query, None).await
    }

    pub(crate) async fn submit_draft(&self, id: StringId) -> Result<UpdateResult, Error> {
        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };
        let update = doc! {
            "$set": {
                "created": Some(Utc::now().timestamp()),
                "state": RecordState::Created,
            }
        };

        self.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub(crate) async fn client_time_trace(
        &self,
        id: StringId,
        time: i64,
        target: TimeTraceFor,
    ) -> Result<UpdateResult, Error> {
        let query = doc! {
            "_id": id.to_object_id()?,
            "state": match target {
                TimeTraceFor::ClientInsideForCollect => RecordState::Created,                   // Client can collected only after request created
                TimeTraceFor::ClientOutsideAfterCollect => RecordState::CollectClientSignature, // Client can go out after collect only after signature
                TimeTraceFor::ClientInsideForReturn => RecordState::CollectPqrsSignature,       // Client can return products only after pqrs signature during collect
                TimeTraceFor::ClientOutsideAfterReturn => RecordState::ReturnClientSignature,   // Client can go out after return only after signature
            },
        };
        let update = match target {
            TimeTraceFor::ClientInsideForCollect => doc! {
                "$set": {
                    "traces": {
                        "collected": {
                            "inside": time,
                        }
                    },
                    "state": RecordState::CollectClientInside,
                },
            },
            TimeTraceFor::ClientOutsideAfterCollect => doc! {
                "$set": {
                    "traces.collected.outside": time,
                    "state": RecordState::CollectClientOutside,
                },
            },
            TimeTraceFor::ClientInsideForReturn => doc! {
                "$set": {
                    "traces.returned": {
                        "inside": time,
                    },
                    "state": RecordState::ReturnClientInside,
                },
            },
            TimeTraceFor::ClientOutsideAfterReturn => doc! {
                "$set": {
                    "traces.returned.outside": time,
                    "state": RecordState::ReturnClientOutside,
                },
            },
        };

        self.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub(crate) async fn signature_trace(
        &self,
        id: StringId,
        signer: Signer,
        target: SignatureTraceFor,
    ) -> Result<UpdateResult, Error> {
        let signer = to_bson(&signer)?;

        let query = doc! {
            "_id": id.to_object_id()?,
            "state": match target {
                SignatureTraceFor::CollectByClient => RecordState::CollectClientInside, // Client can sign only inside office
                SignatureTraceFor::CollectConfirmedByPqrs => RecordState::CollectClientOutside,  // PQRS can sign only after client go out
                SignatureTraceFor::ReturnByClient => RecordState::ReturnClientInside,   // Client can sign only inside office
                SignatureTraceFor::ReturnConfirmedByPqrs => RecordState::ReturnClientOutside,    // PQRS can sign only after client go out
            },
        };

        let update = match target {
            SignatureTraceFor::CollectByClient => doc! {
                "$set": {
                    "traces.collected.client":  signer,
                    "state": RecordState::CollectClientSignature,
                }
            },
            SignatureTraceFor::CollectConfirmedByPqrs => doc! {
                "$set": {
                    "traces.collected.pqrs": signer,
                    "state": RecordState::CollectPqrsSignature,
                }
            },
            SignatureTraceFor::ReturnByClient => doc! {
                "$set": {
                    "traces.returned.client": signer,
                    "state": RecordState::ReturnClientSignature,
                }
            },
            SignatureTraceFor::ReturnConfirmedByPqrs => doc! {
                "$set": {
                    "traces.returned.pqrs": signer,
                    "state": RecordState::ReturnPqrsSignature,
                }
            },
        };

        self.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub(crate) async fn completed(&self, id: StringId) -> Result<UpdateResult, Error> {
        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::ReturnPqrsSignature,
        };
        let update = doc! {
            "$set": {
                "state": RecordState::Completed,
            }
        };

        self.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub(crate) async fn watch(&self) -> Result<ChangeStream<ChangeStreamEvent<Record>>, Error> {
        // max_await_time will have an impact on next_if_any.
        // be careful as this time will impact when a stream/connection will be closed when a client request is closed.
        let options = ChangeStreamOptions::builder()
            .full_document(Some(FullDocumentType::UpdateLookup))
            .max_await_time(Some(Duration::from_secs(5)))
            .build();

        self.register.watch(None, Some(options)).await
    }

    pub(crate) async fn search_by_id(&self, id: StringId) -> Result<Option<Record>, Error> {
        let filter = doc! {
            "_id": id.to_object_id()?,
        };

        self.register.find_one(filter, None).await
    }

    pub(crate) async fn search(
        &self,
        states: Vec<RecordState>,
        range: Option<(i64, i64)>,
    ) -> Result<Cursor<Record>, Error> {
        let filter = match range {
            None => doc! {
                "state": { "$in": states }
            },
            Some(range) => doc! {
                "$and" : vec![
                    doc! {"state": { "$in": states }},
                    doc! {"created": { "$gte": range.0 }},
                    doc! {"created": { "$lte": range.1 }},
                ]
            },
        };

        self.register.find(filter, None).await
    }

    fn error_on_update_unmatched(result: UpdateResult) -> Result<UpdateResult, Error> {
        match result.matched_count {
            0 => Err(Error::custom("no document updated !")),
            _ => Ok(result),
        }
    }
}
