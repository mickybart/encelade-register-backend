mod register_types;
mod string_id;
mod traces_for;

pub use self::register_types::{Record, RecordState, Signer, Trace};
pub use self::string_id::StringId;
pub use self::traces_for::{SignatureTraceFor, TimeTraceFor};

use std::env;
use std::time::Duration;

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

pub struct Mongo {
    pub register: Collection<Record>,
}

static API_VERSION_1: i32 = 1;
static DATABASE_NAME: &'static str = "encelade";
static COLLECTION_NAME: &'static str = "register";

impl Mongo {
    pub async fn init() -> Result<Self, Error> {
        let uri = env::var("MONGODB_URI").map_err(|_| {
            Error::custom("Backend is not ready as MONGODB_URI environment variable is unset !")
        })?;

        let client_options = ClientOptions::parse(uri).await?;

        let client = Client::with_options(client_options)?;

        let register = client.database(DATABASE_NAME).collection(COLLECTION_NAME);

        Ok(Mongo { register })
    }

    pub async fn insert_draft(summary: String) -> Result<InsertOneResult, Error> {
        let db = Mongo::init().await?;

        let draft = Record {
            id: None,
            api_version: API_VERSION_1,
            created: None,
            summary: summary,
            traces: None,
            state: RecordState::Draft,
        };

        db.register.insert_one(draft, None).await
    }

    pub async fn update_draft(id: StringId, summary: String) -> Result<UpdateResult, Error> {
        let db = Mongo::init().await?;

        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };
        let update = doc! {
            "$set": {
                "summary": summary,
            },
        };

        db.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub async fn delete_draft(id: StringId) -> Result<DeleteResult, Error> {
        let db = Mongo::init().await?;

        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };

        db.register.delete_one(query, None).await
    }

    pub async fn submit_draft(id: StringId) -> Result<UpdateResult, Error> {
        let db = Mongo::init().await?;

        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::Draft,
        };
        let update = doc! {
            "$set": {
                "created": Some(0), // TODO: use current date
                "state": RecordState::Created,
            }
        };

        db.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub async fn client_time_trace(
        id: StringId,
        time: i64,
        target: TimeTraceFor,
    ) -> Result<UpdateResult, Error> {
        let db = Mongo::init().await?;

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

        db.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub async fn signature_trace(
        id: StringId,
        signer: Signer,
        target: SignatureTraceFor,
    ) -> Result<UpdateResult, Error> {
        let db = Mongo::init().await?;

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

        db.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub async fn completed(id: StringId) -> Result<UpdateResult, Error> {
        let db = Mongo::init().await?;

        let query = doc! {
            "_id": id.to_object_id()?,
            "state": RecordState::ReturnPqrsSignature,
        };
        let update = doc! {
            "$set": {
                "state": RecordState::Completed,
            }
        };

        db.register
            .update_one(query, update, None)
            .await
            .and_then(Mongo::error_on_update_unmatched)
    }

    pub async fn watch() -> Result<ChangeStream<ChangeStreamEvent<Record>>, Error> {
        let db = Mongo::init().await?;

        // max_await_time will have an impact on next_if_any.
        // be careful as this time will impact when a stream/connection will be closed when a client request is closed.
        let options = ChangeStreamOptions::builder()
            .full_document(Some(FullDocumentType::UpdateLookup))
            .max_await_time(Some(Duration::from_secs(5)))
            .build();

        db.register.watch(None, Some(options)).await
    }

    pub async fn search_by_id(id: StringId) -> Result<Option<Record>, Error> {
        let db = Mongo::init().await?;

        let filter = doc! {
            "_id": id.to_object_id()?,
        };

        db.register.find_one(filter, None).await
    }

    pub async fn search(states: Vec<RecordState>) -> Result<Cursor<Record>, Error> {
        let db = Mongo::init().await?;

        let filter = doc! {
            "state": { "$in": states }
        };

        db.register.find(filter, None).await
    }

    fn error_on_update_unmatched(result: UpdateResult) -> Result<UpdateResult, Error> {
        match result.matched_count {
            0 => Err(Error::custom("no document updated !")),
            _ => Ok(result),
        }
    }
}
