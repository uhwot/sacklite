use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, Path},
    Responder, Result,
};
use maud::html as xml;
use sqlx::{Pool, Postgres};

use crate::responder::Xml;

pub async fn slot(
    path: Path<(String, i64)>,
    pool: Data<Arc<Pool<Postgres>>>,
) -> Result<impl Responder> {
    let (slot_type, id) = path.into_inner();

    // TODO: add support for dev slots

    // https://stackoverflow.com/a/26727307
    let slot = sqlx::query!(
        "SELECT slots.*, author.online_id as author_oid
        FROM slots JOIN users author ON slots.author = author.id
        WHERE slots.id = $1",
        id
    )
    .fetch_optional(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or(error::ErrorNotFound("Slot not found"))?;

    Ok(Xml(xml!(
        slot type="user" {
            id { (slot.id) }
            npHandle { (slot.author_oid) }
            location {
                x { (slot.location_x) }
                y { (slot.location_y) }
            }
            game { (slot.gamever) }
            name { (slot.name) }
            description { (slot.description) }
            rootLevel { (slot.root_level) }
            icon { (slot.icon) }
            initiallyLocked { (slot.initially_locked) }
            isSubLevel { (slot.is_sub_level) }
            isLBP1Only { (slot.is_lbp1_only) }
            shareable { (slot.shareable) }
            minPlayers { (slot.min_players) }
            maxPlayers { (slot.max_players) }
            heartCount { "0" }
            thumbsup { "0" }
            thumbsdown { "0" }
            averageRating { "0.0" } // lbp1
            playerCount { "0" }
            matchingPlayers { "0" }
            mmpick { (slot.mmpicked_at.is_some()) }
            yourRating { "0" } // lbp1
            yourDPadRating { "0" } // lbp2+
            yourlbp1PlayCount { "0" }
            yourlbp2PlayCount { "0" }
            reviewCount { "0" }
            commentCount { "0" }
            photoCount { "0" }
            authorPhotoCount { "0" }
            labels {}
            firstPublished { (slot.published_at.timestamp_millis()) }
            lastUpdated { (slot.updated_at.timestamp_millis()) }
            commentsEnabled { "true" }
            reviewsEnabled { "true" }
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
    ).into_string()))
}