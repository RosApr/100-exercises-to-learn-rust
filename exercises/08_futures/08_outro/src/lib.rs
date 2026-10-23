use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TicketId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    ToDo,
    InProgress,
    Done,
}

#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: TicketId,
    pub title: String,
    pub description: String,
    pub status: Status,
}

#[derive(Debug, Deserialize)]
pub struct TicketDraft {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct TicketPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<Status>,
}

#[derive(Default)]
pub struct TicketStore {
    tickets: BTreeMap<TicketId, Ticket>,
    counter: u64,
}

impl TicketStore {
    fn add(&mut self, draft: TicketDraft) -> Ticket {
        let id = TicketId(self.counter);
        self.counter += 1;
        let ticket = Ticket {
            id,
            title: draft.title,
            description: draft.description,
            status: Status::ToDo,
        };
        self.tickets.insert(id, ticket.clone());
        ticket
    }

    fn get(&self, id: TicketId) -> Option<Ticket> {
        self.tickets.get(&id).cloned()
    }

    fn patch(&mut self, id: TicketId, patch: TicketPatch) -> Option<Ticket> {
        let ticket = self.tickets.get_mut(&id)?;
        if let Some(title) = patch.title {
            ticket.title = title;
        }
        if let Some(description) = patch.description {
            ticket.description = description;
        }
        if let Some(status) = patch.status {
            ticket.status = status;
        }
        Some(ticket.clone())
    }
}

type SharedStore = Arc<RwLock<TicketStore>>;

async fn create_ticket(
    State(store): State<SharedStore>,
    Json(draft): Json<TicketDraft>,
) -> (StatusCode, Json<Ticket>) {
    let ticket = store.write().unwrap().add(draft);
    (StatusCode::CREATED, Json(ticket))
}

async fn get_ticket(
    State(store): State<SharedStore>,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>, StatusCode> {
    store
        .read()
        .unwrap()
        .get(TicketId(id))
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn patch_ticket(
    State(store): State<SharedStore>,
    Path(id): Path<u64>,
    Json(patch): Json<TicketPatch>,
) -> Result<Json<Ticket>, StatusCode> {
    store
        .write()
        .unwrap()
        .patch(TicketId(id), patch)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub fn app() -> Router {
    let store: SharedStore = Arc::new(RwLock::new(TicketStore::default()));
    Router::new()
        .route("/tickets", post(create_ticket))
        .route("/tickets/{id}", get(get_ticket).patch(patch_ticket))
        .with_state(store)
}
