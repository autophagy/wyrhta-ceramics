use axum::response::IntoResponse;
use axum::{extract::State, Json};
use chrono::NaiveDateTime;
use sqlx::sqlite::SqlitePool;

use crate::internal_error;
use crate::models::{ApiResource, Event, State as WorkState};

#[derive(sqlx::FromRow)]
struct EventDTO {
    id: i32,
    work_id: i32,
    previous_state_id: Option<i32>,
    current_state_id: i32,
    created_at: NaiveDateTime,
}

impl From<EventDTO> for Event {
    fn from(event: EventDTO) -> Self {
        Event {
            id: event.id,
            work: (ApiResource::Work, event.work_id).into(),
            previous_state: event.previous_state_id.map(WorkState::from),
            current_state: event.current_state_id.into(),
            created_at: event.created_at,
        }
    }
}

pub(crate) async fn events(State(pool): State<SqlitePool>) -> impl IntoResponse {
    sqlx::query_as::<_, EventDTO>(
        "SELECT e.id, e.work_id, s1.id AS previous_state_id,
        s2.id AS current_state_id, e.created_at
        FROM events e
        LEFT JOIN states s1 ON e.previous_state = s1.id
        LEFT JOIN states s2 ON e.current_state = s2.id
        JOIN works w ON e.work_id = w.id
        ORDER BY e.created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map(|events| events.into_iter().map(Event::from).collect::<Vec<Event>>())
    .map(Json)
    .map_err(internal_error)
}
