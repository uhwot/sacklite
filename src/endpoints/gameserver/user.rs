use axum::{
    routing::{get, post},
    Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::StatusCode,
    Extension
};
use axum_extra::extract::Query;
use futures::TryStreamExt;
use maud::html as xml;
use serde::{Deserialize, Serialize};

use crate::{
    extractors::Xml,
    types::{GameVersion, SessionData, ResourceRef},
    utils::{resource::get_hash_path, serde::double_option_err, db::db_error},
    AppState,
    extractors::Json,
};

use super::Location;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/:online_id", get(user))
        .route("/users", get(users))
        .route("/updateUser", post(update_user))
        .route("/get_my_pins", get(get_my_pins))
        .route("/update_my_pins", post(update_my_pins))
        .route("/privacySettings", get(privacy_settings))
        .route("/privacySettings", post(privacy_settings))
}

async fn user(
    State(state): State<AppState>,
    Path(online_id): Path<String>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    // https://stackoverflow.com/a/26727307
    let user = sqlx::query!(
        "SELECT users.*,
        COUNT(DISTINCT comments.id) AS comment_count,
        COUNT(DISTINCT lbp1slot.id) AS lbp1slot_count,
        COUNT(DISTINCT lbp2slot.id) AS lbp2slot_count,
        COUNT(DISTINCT lbp3slot.id) AS lbp3slot_count,
        COUNT(DISTINCT favourite_slots.slot_id) AS favourite_slot_count
        FROM users
        LEFT JOIN comments ON users.id = comments.target_user
        LEFT JOIN slots lbp1slot ON users.id = lbp1slot.author AND lbp1slot.gamever = 0
        LEFT JOIN slots lbp2slot ON users.id = lbp2slot.author AND lbp2slot.gamever = 1
        LEFT JOIN slots lbp3slot ON users.id = lbp3slot.author AND lbp3slot.gamever = 2
        LEFT JOIN favourite_slots ON users.id = favourite_slots.user_id
        WHERE online_id = $1
        GROUP BY users.id",
        online_id
    )
        .fetch_optional(&state.pool)
        .await
        .map_err(db_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found").into_response())?;

    let slot_limit = state.config.slot_limit as i64;
    let lbp1slot_count = user.lbp1slot_count.unwrap_or_default();
    let lbp2slot_count = user.lbp2slot_count.unwrap_or_default();
    let lbp3slot_count = user.lbp3slot_count.unwrap_or_default();

    Ok(Xml(xml!(
        user type="user" {
            npHandle icon=(user.icon.as_deref().unwrap_or_default()) { (user.online_id) }
            game { (&(session.game_version as u8)) }
            lbp1UsedSlots { (lbp1slot_count) }
            entitledSlots { (slot_limit) }
            freeSlots { (&(slot_limit - lbp1slot_count)) }
            crossControlUsedSlots { "0" }
            crossControlEntitledSlots { (slot_limit) }
            crossControlPurchsedSlots { "0" }
            crossControlFreeSlots { (slot_limit) }
            lbp2UsedSlots { (lbp2slot_count) }
            lbp2EntitledSlots { (slot_limit) }
            lbp2PurchasedSlots { "0" }
            lbp2FreeSlots { (&(slot_limit - lbp2slot_count)) }
            lbp3UsedSlots { (lbp3slot_count) }
            lbp3EntitledSlots { (slot_limit) }
            lbp3PurchasedSlots { "0" }
            lbp3FreeSlots { (&(slot_limit - lbp3slot_count)) }
            lists { "0" }
            lists_quota { "20" }
            heartCount { "0" }
            planets {(
                match session.game_version {
                    GameVersion::Lbp1 => "",
                    GameVersion::Lbp2 => user.lbp2_planets.as_deref().unwrap_or_default(),
                    GameVersion::Lbp3 => user.lbp3_planets.as_deref().unwrap_or_default(),
                })
            }
            crossControlPlanet { (user.cross_control_planet.as_deref().unwrap_or_default()) }
            yay2 { (user.yay2.as_deref().unwrap_or_default()) }
            boo2 { (user.boo2.as_deref().unwrap_or_default()) }
            biography { (user.biography) }
            reviewCount { "0" }
            commentCount { (user.comment_count.unwrap_or_default()) }
            photosByMeCount { "0" }
            photosWithMeCount { "0" }
            commentsEnabled { "true" }
            location {
                x { (user.location_x) }
                y { (user.location_y) }
            }
            favouriteSlotCount { (user.favourite_slot_count.unwrap_or_default()) }
            favouriteUserCount { "0" }
            lolcatftwCount { "0" } // this is the queue, why the fuck would you do this mm
            pins {
                // https://stackoverflow.com/a/61052611
                @let pins: String = user.profile_pins.iter().map(|&pin| pin.to_string() + ",").collect();
                (pins.strip_suffix(',').unwrap_or_default())
            }
            staffChallengeGoldCount { "0" }
            staffChallengeSilverCount { "0" }
            staffChallengeBronzeCount { "0" }
            photos {} // TODO: make separate photo entity?
            /*clientsConnected {
                lbp1 { "true" }
                lbp2 { "true" }
                lbpme { "true" }
                lbp3ps3 { "true" }
            }*/
        }
    )))
}

#[derive(Deserialize)]
struct UsersQuery {
    u: Vec<String>,
}

async fn users(
    query: Query<UsersQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    let mut users = sqlx::query!(
        "SELECT online_id, icon FROM users WHERE online_id = ANY($1)",
        &query.u
    )
    .fetch(&state.pool);

    Ok(Xml(xml!(
        users {
            @while let Some(user) = users.try_next().await.map_err(db_error)? {
                user type="user" {
                    npHandle icon=(user.icon.as_deref().unwrap_or_default()) { (user.online_id) }
                }
            }
        }
    )))
}

#[derive(Deserialize, Debug)]
struct SlotList {
    #[serde(default)]
    slot: Vec<Slot>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UpdateUserPayload {
    location: Option<Location>,
    biography: Option<String>,
    #[serde(default, with = "double_option_err")]
    icon: Option<Option<ResourceRef>>,
    #[serde(default, with = "double_option_err")]
    planets: Option<Option<ResourceRef>>,
    #[serde(default, with = "double_option_err")]
    cross_control_planet: Option<Option<ResourceRef>>,
    slots: Option<SlotList>,
    #[serde(default, with = "double_option_err")]
    yay2: Option<Option<ResourceRef>>,
    #[serde(default, with = "double_option_err")]
    meh2: Option<Option<ResourceRef>>,
    #[serde(default, with = "double_option_err")]
    boo2: Option<Option<ResourceRef>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Slot {
    id: i64,
    location: Location,
}

async fn update_user(
    State(state): State<AppState>,
    session: Extension<SessionData>,
    payload: Xml<UpdateUserPayload>,
) -> Result<impl IntoResponse, Response> {
    if let Some(Some(icon)) = &payload.icon {
        if !icon.exists(&state.config.resource_dir) {
            return Err((StatusCode::BAD_REQUEST, "Icon resource invalid").into_response());
        }
    }
    for res_ref in [
        &payload.planets,
        &payload.cross_control_planet,
        &payload.yay2,
        &payload.meh2,
        &payload.boo2,
    ].into_iter().flatten().flatten() {
        match res_ref {
            ResourceRef::Hash(hash) => {
                if !get_hash_path(&state.config.resource_dir, *hash).exists() {
                    return Err((StatusCode::BAD_REQUEST, "Resource(s) invalid").into_response());
                }
            },
            ResourceRef::Guid(_) => return Err((StatusCode::BAD_REQUEST, "Resource(s) cannot be a GUID").into_response())
        }
    }

    let uid = session.user_id;

    if let Some(location) = &payload.location {
        sqlx::query!(
            "UPDATE users SET location_x = $1, location_y = $2 WHERE id = $3",
            location.x as i32,
            location.y as i32,
            uid
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(bio) = &payload.biography {
        sqlx::query!("UPDATE users SET biography = $1 WHERE id = $2", bio, uid)
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
    }
    if let Some(icon) = &payload.icon {
        sqlx::query!("UPDATE users SET icon = $1 WHERE id = $2", icon.as_ref().map(|r| r.to_string()), uid)
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
    }
    if let Some(planets) = &payload.planets {
        let planets = planets.as_ref().map(|r| r.to_string());
        match session.game_version {
            GameVersion::Lbp1 => {}
            GameVersion::Lbp2 => {
                sqlx::query!(
                    "UPDATE users SET lbp2_planets = $1 WHERE id = $2",
                    planets,
                    uid
                )
                .execute(&state.pool)
                .await
                .map_err(db_error)?;
            }
            GameVersion::Lbp3 => {
                sqlx::query!(
                    "UPDATE users SET lbp3_planets = $1 WHERE id = $2",
                    planets,
                    uid
                )
                .execute(&state.pool)
                .await
                .map_err(db_error)?;
            }
        }
    }
    if let Some(ccplanet) = &payload.cross_control_planet {
        sqlx::query!(
            "UPDATE users SET cross_control_planet = $1 WHERE id = $2",
            ccplanet.as_ref().map(|r| r.to_string()),
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(yay2) = &payload.yay2 {
        sqlx::query!(
            "UPDATE users SET yay2 = $1 WHERE id = $2",
            yay2.as_ref().map(|r| r.to_string()),
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(meh2) = &payload.meh2 {
        sqlx::query!(
            "UPDATE users SET meh2 = $1 WHERE id = $2",
            meh2.as_ref().map(|r| r.to_string()),
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(boo2) = &payload.boo2 {
        sqlx::query!(
            "UPDATE users SET boo2 = $1 WHERE id = $2",
            boo2.as_ref().map(|r| r.to_string()),
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(slots) = &payload.slots {
        for slot in &slots.slot {
            sqlx::query!(
                "UPDATE slots
                SET location_x = $1, location_y = $2
                WHERE id = $3 AND author = $4",
                slot.location.x as i32, slot.location.y as i32,
                slot.id, session.user_id
            )
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
        }
    }

    Ok(StatusCode::OK)
}

#[derive(Serialize, Deserialize, Clone)]
struct UserPinsPayload {
    progress: Option<Vec<i64>>,
    awards: Option<Vec<i64>>,
    // packet captures don't have profile pins in responses
    #[serde(skip_serializing)]
    profile_pins: Option<Vec<i64>>,
}

async fn get_my_pins(
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    let user = sqlx::query!(
        "SELECT progress, awards FROM users WHERE id = $1",
        session.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found").into_response())?;

    Ok(Json(UserPinsPayload {
        progress: Some(user.progress),
        awards: Some(user.awards),
        profile_pins: None,
    }))
}

async fn update_my_pins(
    State(state): State<AppState>,
    session: Extension<SessionData>,
    payload: Json<UserPinsPayload>,
) -> Result<impl IntoResponse, Response> {
    if let Some(pins) = &payload.profile_pins {
        if pins.len() > 3 {
            return Err((StatusCode::BAD_REQUEST, "Invalid profile pins list").into_response());
        }
    }

    if let Some(awards) = &payload.awards {
        sqlx::query!(
            "UPDATE users SET awards = $1 WHERE id = $2",
            awards,
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(progress) = &payload.progress {
        sqlx::query!(
            "UPDATE users SET progress = $1 WHERE id = $2",
            progress,
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }
    if let Some(pins) = &payload.profile_pins {
        sqlx::query!(
            "UPDATE users SET profile_pins = $1 WHERE id = $2",
            pins,
            session.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    }

    Ok(payload)
}

async fn privacy_settings() -> impl IntoResponse {
    Xml(xml!(
        privacySettings {
            levelVisibility { "all" }
            profileVisibility { "all" }
        }
    ))
}