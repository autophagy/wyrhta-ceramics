use axum::extract::{Json as ExtractJson, Path, State};
use axum::response::IntoResponse;
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::error::{internal_error, Error};
use crate::models::{
    is_valid_transition, ApiResource, Clay, CurrentState, Event, Images, PostWork, PutWork,
    State as WorkState, Work,
};
use crate::result::{EmptyResult, JsonResult, OptionalResult};
use crate::AppState;

pub(crate) static WORK_DTO_QUERY: &str = "
SELECT w.id, w.project_id, w.name, w.notes, w.glaze_description, w.created_at, w.header_key, w.thumbnail_key, w.is_multiple,
e.current_state_id, e.current_state_transitioned,
c.id as clay_id, c.name as clay_name, c.description as clay_description, c.shrinkage as clay_shrinkage
FROM works w
JOIN (
    SELECT work_id, current_state as current_state_id, created_at as current_state_transitioned
    FROM events
    WHERE id IN (
        SELECT MAX(id)
        FROM events
        GROUP BY work_id
    )
) e ON w.id = e.work_id
JOIN clays c ON w.clay_id = c.id";

#[derive(sqlx::FromRow, Serialize)]
pub(crate) struct WorkDTO {
    id: i32,
    project_id: i32,
    name: String,
    notes: Option<String>,
    clay_id: i32,
    clay_name: String,
    clay_description: Option<String>,
    clay_shrinkage: f64,
    current_state_id: i32,
    current_state_transitioned: NaiveDateTime,
    glaze_description: Option<String>,
    header_key: Option<String>,
    thumbnail_key: Option<String>,
    created_at: NaiveDateTime,
    is_multiple: bool,
}

pub(crate) fn workdto_to_work(workdto: WorkDTO, _appstate: &AppState) -> Work {
    let images = Images {
        header: workdto.header_key,
        thumbnail: workdto.thumbnail_key,
    };

    let clay = Clay {
        id: workdto.clay_id,
        name: workdto.clay_name,
        description: workdto.clay_description,
        shrinkage: workdto.clay_shrinkage,
    };

    Work {
        id: workdto.id,
        project: (ApiResource::Project, workdto.project_id).into(),
        name: workdto.name,
        notes: workdto.notes,
        clay,
        current_state: CurrentState {
            state: workdto.current_state_id.into(),
            transitioned_at: workdto.current_state_transitioned,
        },
        glaze_description: workdto.glaze_description,
        images,
        created_at: workdto.created_at,
        is_multiple: workdto.is_multiple,
    }
}

pub(crate) async fn works(State(appstate): State<AppState>) -> JsonResult<Vec<Work>> {
    sqlx::query_as::<_, WorkDTO>(WORK_DTO_QUERY)
        .fetch_all(&appstate.pool)
        .await
        .map(|works| {
            works
                .into_iter()
                .map(|w| workdto_to_work(w, &appstate))
                .collect::<Vec<Work>>()
        })
        .into()
}

pub(crate) async fn work(
    Path(id): Path<i32>,
    State(appstate): State<AppState>,
) -> OptionalResult<Work> {
    sqlx::query_as::<_, WorkDTO>(&format!("{} {}", WORK_DTO_QUERY, "WHERE w.id = ?"))
        .bind(id)
        .fetch_optional(&appstate.pool)
        .await
        .map(|work_dto| work_dto.map(|w| workdto_to_work(w, &appstate)))
        .into()
}

#[derive(sqlx::FromRow, Debug)]
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

pub(crate) async fn events(
    Path(id): Path<i32>,
    State(appstate): State<AppState>,
) -> JsonResult<Vec<Event>> {
    sqlx::query_as::<_, EventDTO>(
        "SELECT e.id, e.work_id, s1.id AS previous_state_id,
        s2.id AS current_state_id, e.created_at
        FROM events e
        LEFT JOIN states s1 ON e.previous_state = s1.id
        LEFT JOIN states s2 ON e.current_state = s2.id
        WHERE e.work_id = ?",
    )
    .bind(id)
    .fetch_all(&appstate.pool)
    .await
    .map(|events| events.into_iter().map(Event::from).collect::<Vec<Event>>())
    .into()
}

// PUT

pub(crate) async fn put_work(
    Path(id): Path<i32>,
    State(appstate): State<AppState>,
    ExtractJson(data): ExtractJson<PutWork>,
) -> EmptyResult {
    sqlx::query(
        "UPDATE works
        SET project_id=?, name=?, notes=?, clay_id=?, glaze_description=?,
        header_key=?, thumbnail_key=?, is_multiple=?
        WHERE id=?",
    )
    .bind(data.project_id)
    .bind(data.name)
    .bind(data.notes)
    .bind(data.clay_id)
    .bind(data.glaze_description)
    .bind(data.header)
    .bind(data.thumbnail)
    .bind(data.is_multiple)
    .bind(id)
    .execute(&appstate.pool)
    .await
    .into()
}

pub(crate) async fn put_state(
    Path(id): Path<i32>,
    State(appstate): State<AppState>,
    ExtractJson(data): ExtractJson<WorkState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let latest_event = sqlx::query_as::<_, EventDTO>(
        "SELECT id, work_id, previous_state as previous_state_id, current_state as current_state_id, created_at
        FROM events
        WHERE work_id = ?
        ORDER BY created_at DESC
        LIMIT 1"
    )
        .bind(id)
        .fetch_one(&appstate.pool)
        .await
        .map_err(internal_error)?;

    let current_state = WorkState::from(latest_event.current_state_id);
    if is_valid_transition(current_state.clone(), data.clone()) {
        let new_previous_state_id: &i32 = &current_state.into();
        let new_current_state_id: &i32 = &data.into();

        sqlx::query(
            "INSERT INTO events (work_id, previous_state, current_state)
            VALUES (?, ?, ?)",
        )
        .bind(id)
        .bind(new_previous_state_id)
        .bind(new_current_state_id)
        .execute(&appstate.pool)
        .await
        .map(|_| ())
        .map_err(internal_error)
    } else {
        Err(Error::InvalidStateTransition)
    }
}

// POST

async fn insert_work(appstate: &AppState, post_work: &PostWork) -> Result<i32, sqlx::Error> {
    let initial_state_id: &i32 = &post_work.state.clone().into();

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO works (project_id, name, notes, clay_id, glaze_description, header_key, thumbnail_key, is_multiple)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id"
    )
    .bind(post_work.project_id)
    .bind(&post_work.name)
    .bind(&post_work.notes)
    .bind(post_work.clay_id)
    .bind(&post_work.glaze_description)
    .bind(&post_work.header)
    .bind(&post_work.thumbnail)
    .bind(&post_work.is_multiple)
    .fetch_one(&appstate.pool)
    .await?;

    sqlx::query(
        "INSERT INTO events (work_id, current_state)
        VALUES (?, ?)",
    )
    .bind(id)
    .bind(initial_state_id)
    .execute(&appstate.pool)
    .await?;

    Ok(id)
}

pub(crate) async fn post_work(
    State(appstate): State<AppState>,
    ExtractJson(data): ExtractJson<PostWork>,
) -> JsonResult<i32> {
    insert_work(&appstate, &data).await.into()
}

// DELETE

async fn delete_work_and_events(appstate: &AppState, id: &i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM events WHERE work_id = ?")
        .bind(id)
        .execute(&appstate.pool)
        .await?;

    sqlx::query("DELETE FROM works WHERE id = ?")
        .bind(id)
        .execute(&appstate.pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_work(
    Path(id): Path<i32>,
    State(appstate): State<AppState>,
) -> EmptyResult {
    delete_work_and_events(&appstate, &id).await.into()
}
