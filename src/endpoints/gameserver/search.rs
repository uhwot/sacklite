use axum::{Router, routing::get, extract::{State, Path}, Extension, response::{IntoResponse, Response}};
use axum_extra::extract::Query;
use chrono::NaiveDateTime;
use maud::html as xml;
use serde::Deserialize;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{extractors::Xml, types::SessionData, AppState, utils::db::{db_error, get_id_from_username}};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/slots", get(slots_newest))
        .route("/slots/by", get(slots_by))
        .route("/favouriteSlots/:username", get(favourite_slots))
        .route("/slots/lolcatftw/:username", get(queued_slots))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlotSearchQuery {
    page_start: i64,
    page_size: i64,
    // TODO: game_filter_type: Option<String>,
}

#[derive(sqlx::FromRow)]
struct Slot {
    id: i64,
    name: String,
    author_name: String,
    gamever: i16,
    heart_count: i64,
    mmpicked_at: Option<NaiveDateTime>,
    description: String,
    icon: String,
    root_level: String,
    resources: Vec<String>,
    location_x: i32,
    location_y: i32,
    initially_locked: bool,
    is_sub_level: bool,
    is_lbp1_only: bool,
    shareable: bool,
    // TODO: level_type: String,
    //labels: Option<Vec<String>>,
    // TODO: move_required: bool,
    // TODO: vita_cc_required: bool,

    total: i64,
}

enum SlotSearchFilter {
    Newest,
    UploadedBy(Uuid),
    LastHeartedBy(Uuid),
    LastQueuedBy(Uuid),
}

async fn slots_newest(
    query: Query<SlotSearchQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    slot_search(
        state, session,
        SlotSearchFilter::Newest,
        query
    ).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlotsByQuery {
    u: String,
}

async fn slots_by(
    query: Query<SlotSearchQuery>,
    query2: Query<SlotsByQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    let uploaded_by = get_id_from_username(&query2.u, &state).await?;
    slot_search(
        state, session,
        SlotSearchFilter::UploadedBy(uploaded_by),
        query
    ).await
}

async fn favourite_slots(
    query: Query<SlotSearchQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
    Path(hearted_by): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let hearted_by = get_id_from_username(&hearted_by, &state).await?;
    slot_search(
        state, session,
        SlotSearchFilter::LastHeartedBy(hearted_by),
        query
    ).await
}

async fn queued_slots(
    query: Query<SlotSearchQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
    Path(queued_by): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let queued_by = get_id_from_username(&queued_by, &state).await?;
    slot_search(
        state, session,
        SlotSearchFilter::LastQueuedBy(queued_by),
        query
    ).await
}

async fn slot_search(
    state: AppState,
    session: Extension<SessionData>,
    filter: SlotSearchFilter,
    query: Query<SlotSearchQuery>,
) -> Result<impl IntoResponse, Response> {
    // well, this is horrifying ¯\_(ツ)_/¯
    let mut sql = QueryBuilder::new(
        "SELECT slots.*,
        users.online_id AS author_name,
        count(DISTINCT hearts.user_id) AS heart_count,
        count(slots.id) OVER() AS total
        FROM slots
        JOIN users ON slots.author = users.id
        LEFT JOIN favourite_slots AS hearts ON slots.id = hearts.slot_id"
    );

    if let SlotSearchFilter::LastHeartedBy(user_id) = filter {
        sql.push(" JOIN favourite_slots AS own_hearts ON slots.id = own_hearts.slot_id AND own_hearts.user_id = ");
        sql.push_bind(user_id);
    }
    if let SlotSearchFilter::LastQueuedBy(user_id) = filter {
        sql.push(" JOIN queued_slots AS own_queues ON slots.id = own_queues.slot_id AND own_queues.user_id = ");
        sql.push_bind(user_id);
    }

    sql.push(" WHERE gamever <= ");
    sql.push_bind(session.game_version as i16);
    sql.push(" AND (is_sub_level = FALSE OR author = ");
    sql.push_bind(session.user_id);
    sql.push(')');
    if let SlotSearchFilter::UploadedBy(user_id) = filter {
        sql.push(" AND author = ");
        sql.push_bind(user_id);
    }
    sql.push(" GROUP BY slots.id, author_name");

    if let SlotSearchFilter::LastHeartedBy(_) = filter {
        sql.push(", own_hearts.timestamp");
    }
    if let SlotSearchFilter::LastQueuedBy(_) = filter {
        sql.push(", own_queues.timestamp");
    }

    sql.push(" ORDER BY ");
    sql.push(match filter {
        SlotSearchFilter::Newest => "published_at DESC",
        SlotSearchFilter::UploadedBy(_) => "published_at DESC",
        SlotSearchFilter::LastHeartedBy(_) => "own_hearts.timestamp DESC",
        SlotSearchFilter::LastQueuedBy(_) => "own_queues.timestamp DESC",
    });
    sql.push(" LIMIT ");
    sql.push_bind(query.page_size);
    sql.push(" OFFSET ");
    sql.push_bind(query.page_start - 1);

    let slots = sql.build_query_as::<Slot>()
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?;
    
    let total = slots.first().map_or(0, |s| s.total);
    let hint_start = slots.len() + 1;

    let xml_list = xml!(
        @for slot in slots {
            slot type="user" {
                id { (slot.id) }
                npHandle { (slot.author_name) }
                location {
                    x { (slot.location_x) }
                    y { (slot.location_y) }
                }
                game { (slot.gamever) }
                name { (slot.name) }
                description { (slot.description) }
                rootLevel { (slot.root_level) }
                @for resource in slot.resources {
                    resource { (resource) }
                }
                icon { (slot.icon) }
                initiallyLocked { (slot.initially_locked) }
                isSubLevel { (slot.is_sub_level) }
                isLBP1Only { (slot.is_lbp1_only) }
                shareable { (slot.shareable) }
                heartCount { (slot.heart_count) }
                thumbsup { "0" }
                thumbsdown { "0" }
                averageRating { "0.0" } // lbp1
                playerCount { "0" }
                matchingPlayers { "0" }
                mmpick { (slot.mmpicked_at.is_some()) }
                isAdventurePlanet { "false" } // lbp3
                ps4Only { "false" } // lbp3
                playCount { "0" } // all games
                completionCount { "0" } // all games
                lbp1PlayCount { "0" }
                lbp1CompletionCount { "0" }
                lbp1UniquePlayCount { "0" }
                lbp2PlayCount { "0" }
                lbp2CompletionCount { "0" }
                uniquePlayCount { "0" }
                lbp3PlayCount { "0" }
                lbp3CompletionCount { "0" }
                lbp3UniquePlayCount { "0" }
            }
        }
    );

    // /slots and /favouriteSlots endpoints return different root element names
    Ok(Xml(xml!(
        @match filter {
            SlotSearchFilter::LastHeartedBy(_) => { favouriteSlots total=(total) hint_start=(hint_start) {(xml_list)} },
            _ => { slots total=(total) hint_start=(hint_start) {(xml_list)} },
        }
    )))
}