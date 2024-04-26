pub use internal::register_server::RegisterServer;

use internal::{Draft, Record, RecordId, SearchRequest, SignerTrace, TimestampTrace, Traces};
use num_traits::FromPrimitive;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status};

use crate::mongodb as db;

mod internal {
    tonic::include_proto!("register");
}

#[derive(Debug, Default)]
pub struct Register {}

#[tonic::async_trait]
impl internal::register_server::Register for Register {
    async fn new_draft(&self, request: Request<Draft>) -> Result<Response<RecordId>, Status> {
        let request = request.into_inner();

        db::Mongo::insert_draft(request.summary)
            .await
            .map(|result| {
                Response::new(RecordId {
                    id: result.inserted_id.to_string(),
                })
            })
            .map_err(|e| Status::aborted(e.to_string()))
    }

    async fn update_draft(&self, request: Request<Draft>) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        db::Mongo::update_draft(db::StringId(request.id), request.summary)
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }

    async fn delete_draft(&self, request: Request<RecordId>) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        db::Mongo::delete_draft(db::StringId(request.id))
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }

    async fn submit_draft(&self, request: Request<RecordId>) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        db::Mongo::submit_draft(db::StringId(request.id))
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }

    async fn collect_client_inside(
        &self,
        request: Request<TimestampTrace>,
    ) -> Result<Response<()>, Status> {
        Register::client_time_trace(request, db::TimeTraceFor::ClientInsideForCollect).await
    }

    async fn collect_client_signature(
        &self,
        request: Request<SignerTrace>,
    ) -> Result<Response<()>, Status> {
        Register::signature_trace(request, db::SignatureTraceFor::CollectByClient).await
    }

    async fn collect_client_outside(
        &self,
        request: Request<TimestampTrace>,
    ) -> Result<Response<()>, Status> {
        Register::client_time_trace(request, db::TimeTraceFor::ClientOutsideAfterCollect).await
    }

    async fn collect_pqrs_signature(
        &self,
        request: Request<SignerTrace>,
    ) -> Result<Response<()>, Status> {
        Register::signature_trace(request, db::SignatureTraceFor::CollectConfirmedByPqrs).await
    }

    async fn return_client_inside(
        &self,
        request: Request<TimestampTrace>,
    ) -> Result<Response<()>, Status> {
        Register::client_time_trace(request, db::TimeTraceFor::ClientInsideForReturn).await
    }

    async fn return_client_signature(
        &self,
        request: Request<SignerTrace>,
    ) -> Result<Response<()>, Status> {
        Register::signature_trace(request, db::SignatureTraceFor::ReturnByClient).await
    }

    async fn return_client_outside(
        &self,
        request: Request<TimestampTrace>,
    ) -> Result<Response<()>, Status> {
        Register::client_time_trace(request, db::TimeTraceFor::ClientOutsideAfterReturn).await
    }

    async fn return_pqrs_signature(
        &self,
        request: Request<SignerTrace>,
    ) -> Result<Response<()>, Status> {
        Register::signature_trace(request, db::SignatureTraceFor::ReturnConfirmedByPqrs).await
    }

    async fn complete(&self, request: Request<RecordId>) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        db::Mongo::completed(db::StringId(request.id))
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }

    type SearchStream = ReceiverStream<Result<Record, Status>>;

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let request = request.into_inner();

        let states = request
            .states
            .iter()
            .map(|i| FromPrimitive::from_i32(i.to_owned()).unwrap_or(db::RecordState::Unspecified))
            .collect();

        let mut cursor = db::Mongo::search(states)
            .await
            .map_err(|e| Status::aborted(e.to_string()))?;

        let (tx, rx) = mpsc::channel::<Result<Record, Status>>(10);

        tokio::spawn(async move {
            while let Some(doc) = cursor.next().await {
                let doc = doc
                    .map(|res| res.into())
                    .map_err(|e| Status::aborted(e.to_string()));

                if let Err(_) = tx.send(doc).await {
                    // TODO: logging error
                    return;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn search_by_id(&self, request: Request<RecordId>) -> Result<Response<Record>, Status> {
        let request = request.into_inner();

        db::Mongo::search_by_id(db::StringId(request.id))
            .await
            .map(|result| {
                Response::new(result.map_or(
                    // No record found with this id
                    Record {
                        id: "".to_owned(),
                        api_version: 0,
                        summary: "".to_owned(),
                        created: None,
                        traces: None,
                        state: 0,
                    },
                    |result| result.into(),
                ))
            })
            .map_err(|e| Status::aborted(e.to_string()))
    }
}

impl Register {
    fn empty_response() -> Response<()> {
        Response::new(())
    }

    async fn client_time_trace(
        request: Request<TimestampTrace>,
        target: db::TimeTraceFor,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        let time = request.time.map_or(0, |time| time.seconds);

        db::Mongo::client_time_trace(db::StringId(request.id), time, target)
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }

    async fn signature_trace(
        request: Request<SignerTrace>,
        target: db::SignatureTraceFor,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        let signer = request
            .signer
            .ok_or(Status::invalid_argument("Signer is required !"))?;

        let signer = db::Signer {
            name: signer.name,
            signature: signer.signature,
        };

        db::Mongo::signature_trace(db::StringId(request.id), signer, target)
            .await
            .map(|_| Register::empty_response())
            .map_err(|e| Status::aborted(e.to_string()))
    }
}

impl From<crate::mongodb::Record> for internal::Record {
    fn from(value: crate::mongodb::Record) -> Self {
        let created = value.created.map(|created| Timestamp {
            seconds: created,
            nanos: 0,
        });
        let traces = value.traces.map(|traces| Traces {
            collected: traces.collected.map(|trace| trace.into()),
            returned: traces.returned.map(|trace| trace.into()),
        });

        Self {
            id: value.id.unwrap().to_string(),
            api_version: value.api_version,
            created,
            summary: value.summary,
            traces,
            state: value.state as i32,
        }
    }
}

impl From<crate::mongodb::Trace> for internal::Trace {
    fn from(value: crate::mongodb::Trace) -> Self {
        let to_timestamp = |time: i64| Timestamp {
            seconds: time,
            nanos: 0,
        };
        let to_signer = |signer: db::Signer| internal::Signer {
            name: signer.name,
            signature: signer.signature,
        };

        let inside = value.inside.map(to_timestamp);
        let outside = value.outside.map(to_timestamp);
        let client = value.client.map(to_signer);
        let pqrs = value.pqrs.map(to_signer);

        Self {
            inside,
            outside,
            client,
            pqrs,
        }
    }
}
