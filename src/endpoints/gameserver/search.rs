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
        .route("/slots/:route", get(slots_newest))
        .route("/favouriteSlots/:username", get(favourite_slots))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlotSearchQuery {
    u: Option<String>,
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

enum SlotSearchOrder {
    Newest,
    LastHearted,
}

async fn slots_newest(
    query: Query<SlotSearchQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    slot_search(
        query, state, session,
        SlotSearchOrder::Newest,
        None
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
        query, state, session,
        SlotSearchOrder::LastHearted,
        Some(hearted_by)
    ).await
}

async fn slot_search(
    query: Query<SlotSearchQuery>,
    state: AppState,
    session: Extension<SessionData>,
    order: SlotSearchOrder,
    hearted_by: Option<Uuid>,
) -> Result<impl IntoResponse, Response> {
    let user_id = match &query.u {
        Some(username) => Some(get_id_from_username(username, &state).await?),
        None => None,
    };

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
    if hearted_by.is_some() {
        sql.push(" JOIN favourite_slots AS own_hearts ON slots.id = own_hearts.slot_id AND own_hearts.user_id = ");
        sql.push_bind(hearted_by);
    }
    sql.push(" WHERE gamever <= ");
    sql.push_bind(session.game_version as i16);
    if let Some(user_id) = user_id {
        sql.push(" AND author = ");
        sql.push_bind(user_id);
    }
    sql.push(" GROUP BY slots.id, author_name");
    if hearted_by.is_some() {
        sql.push(", own_hearts.timestamp");
    }
    sql.push(" ORDER BY ");
    sql.push(match order {
        SlotSearchOrder::Newest => "published_at DESC",
        SlotSearchOrder::LastHearted => "own_hearts.timestamp DESC",
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
        @match hearted_by {
            None => { slots total=(total) hint_start=(hint_start) {(xml_list)} },
            Some(_) => { favouriteSlots total=(total) hint_start=(hint_start) {(xml_list)} },
        }
    )))
}