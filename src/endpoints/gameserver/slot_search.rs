use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, ReqData, Query},
    Responder, Result,
};
use maud::html as xml;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{responder::Xml, types::SessionData};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotSearchQuery {
    u: String,
    page_start: u64,
    page_size: u64,
    game_filter_type: Option<String>,
}

pub async fn slots_by(
    query: Query<SlotSearchQuery>,
    pool: Data<Arc<Pool<Postgres>>>,
    session: ReqData<SessionData>,
) -> Result<impl Responder> {
    let user_id = sqlx::query!("SELECT id FROM users WHERE online_id = $1", query.u)
        .fetch_optional(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or(error::ErrorNotFound("User not found"))?
        .id;

    // TODO: use game_filter_type param

    let slots = sqlx::query!(
        "SELECT *, count(*) OVER() AS total
        FROM slots
        WHERE author = $1 AND gamever <= $2
        ORDER BY published_at DESC
        LIMIT $3 OFFSET $4",
        user_id, session.game_version as i16, query.page_size as i64, query.page_start as i64 - 1
    )
    .fetch_all(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?;
    
    let total = slots.get(0);
    let total = match total {
        Some(r) => r.total.unwrap(),
        None => 0,
    };
    let hint_start = slots.len() + 1;

    Ok(Xml(xml!(
        slots total=(total) hint_start=(hint_start) {
            @for slot in slots {
                slot type="user" {
                    id { (slot.id) }
                    npHandle { (query.u) }
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
                    heartCount { "0" }
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
        }
    ).into_string()))
}