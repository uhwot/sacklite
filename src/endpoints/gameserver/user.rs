use std::sync::Arc;

use actix_web::{web::{self, Json, Data, ReqData, Path}, Result, Responder, error, HttpResponse};
use maud::html as xml;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use futures::TryStreamExt;

use crate::{responder::Xml, types::{SessionData, GameVersion, Config, gamever_to_num}, utils::resource::res_exists};

use super::Location;

pub async fn user(path: Path<String>, pool: Data<Arc<Pool<Postgres>>>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let online_id = path.into_inner();

    // https://stackoverflow.com/a/26727307
    let user = sqlx::query!(
        "SELECT users.*, COUNT(comments.id) AS comment_count
        FROM users LEFT JOIN comments ON users.id = comments.target_user
        WHERE online_id = $1
        GROUP BY users.id",
        online_id
    )
    .fetch_optional(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or(error::ErrorNotFound("User not found"))?;

    Ok(Xml(xml!(
        user type="user" {
            npHandle icon=(user.icon) { (user.online_id) }
            game { (gamever_to_num(&session.game_version)) }
            lbp1UsedSlots { "0" }
            entitledSlots { "20" }
            freeSlots { "20" }
            crossControlUsedSlots { "0" }
            crossControlEntitledSlots { "20" }
            crossControlPurchsedSlots { "0" }
            crossControlFreeSlots { "20" }
            lbp2UsedSlots { "0" }
            lbp2EntitledSlots { "20" }
            lbp2PurchasedSlots { "0" }
            lbp2FreeSlots { "20" }
            lbp3UsedSlots { "0" }
            lbp3EntitledSlots { "20" }
            lbp3PurchasedSlots { "0" }
            lbp3FreeSlots { "20" }
            lists { "0" }
            lists_quota { "20" }
            heartCount { "0" }
            planets {(
                match session.game_version {
                    GameVersion::Lbp1 => "",
                    GameVersion::Lbp2 => &user.lbp2_planets,
                    GameVersion::Lbp3 => &user.lbp3_planets
                })
            }
            crossControlPlanet { (user.cross_control_planet) }
            yay2 { (user.yay2) }
            boo2 { (user.boo2) }
            biography { (user.biography) }
            reviewCount { "0" }
            commentCount { (user.comment_count.unwrap_or(0)) }
            photosByMeCount { "0" }
            photosWithMeCount { "0" }
            commentsEnabled { "true" }
            location {
                x { (user.location_x) }
                y { (user.location_y) }
            }
            favouriteSlotCount { "0" }
            favouriteUserCount { "0" }
            lolcatftwCount { "0" } // this is the queue, why the fuck would you do this mm
            pins {
                // https://stackoverflow.com/a/61052611
                @let pins: String = user.profile_pins.iter().map(|&pin| pin.to_string() + ",").collect();
                (pins[..pins.len() - 1])
            }
            staffChallengeGoldCount { "0" }
            staffChallengeSilverCount { "0" }
            staffChallengeBronzeCount { "0" }
            photos {} // TODO: make separate photo entity?
            clientsConnected {
                lbp1 { "true" }
                lbp2 { "true" }
                lbpme { "true" }
                lbp3ps3 { "true" }
            }
        }
    ).into_string()))
}

#[derive(Deserialize)]
pub struct UsersQuery {
    u: Vec<String>,
}

pub async fn users(query: web::Query<UsersQuery>, pool: Data<Arc<Pool<Postgres>>>) -> Result<impl Responder> {
    let mut users = sqlx::query!(
        "SELECT online_id, icon FROM users WHERE online_id = ANY($1)", &query.u[..]
    )
    .fetch(&***pool);

    Ok(Xml(xml!(
        users {
            @while let Some(user) = users.try_next().await.map_err(error::ErrorInternalServerError)? {
                user type="user" {
                    npHandle icon=(user.icon) { (user.online_id) }
                }
            }
        }
    ).into_string()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPayload {
    location: Option<Location>,
    biography: Option<String>,
    icon: Option<String>,
    planets: Option<String>,
    cross_control_planet: Option<String>,
    slots: Option<Vec<Slot>>,
    yay2: Option<String>,
    meh2: Option<String>,
    boo2: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    id: u64,
    location: Location,
}

pub async fn update_user(payload: actix_xml::Xml<UpdateUserPayload>, pool: Data<Arc<Pool<Postgres>>>, session: ReqData<SessionData>, config: Data<Config>) -> Result<impl Responder> {
    if let Some(icon) = &payload.icon {
        if !res_exists(&config.resource_dir, &icon, false, true) {
            return Err(error::ErrorBadRequest("Icon resource invalid"));
        }
    }
    for resource_ref in [
        &payload.planets,
        &payload.cross_control_planet,
        &payload.yay2,
        &payload.meh2,
        &payload.boo2,
    ] {
        if let Some(res_ref) = resource_ref {
            if !res_exists(&config.resource_dir, &res_ref, false, false) {
                return Err(error::ErrorBadRequest("Resource(s) invalid"));
            }
        }
    }

    let uid = session.user_id;

    if let Some(location) = &payload.location {
        sqlx::query!(
            "UPDATE users SET location_x = $1, location_y = $2 WHERE id = $3",
            location.x as i32, location.y as i32, uid
        )
        .execute(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(bio) = &payload.biography {
        sqlx::query!("UPDATE users SET biography = $1 WHERE id = $2", bio, uid)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(icon) = &payload.icon {
        sqlx::query!("UPDATE users SET icon = $1 WHERE id = $2", icon, uid)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(planets) = &payload.planets {
        match session.game_version {
            GameVersion::Lbp1 => {},
            GameVersion::Lbp2 => {
                sqlx::query!(
                    "UPDATE users SET lbp2_planets = $1 WHERE id = $2", planets, uid
                )
                .execute(&***pool)
                .await
                .map_err(error::ErrorInternalServerError)?;
            },
            GameVersion::Lbp3 => {
                sqlx::query!(
                    "UPDATE users SET lbp3_planets = $1 WHERE id = $2", planets, uid
                )
                .execute(&***pool)
                .await
                .map_err(error::ErrorInternalServerError)?;
            },
        }
    }
    if let Some(ccplanet) = &payload.cross_control_planet {
        sqlx::query!("UPDATE users SET cross_control_planet = $1 WHERE id = $2", ccplanet, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(yay2) = &payload.yay2 {
        sqlx::query!("UPDATE users SET yay2 = $1 WHERE id = $2", yay2, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(meh2) = &payload.meh2 {
        sqlx::query!("UPDATE users SET meh2 = $1 WHERE id = $2", meh2, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(boo2) = &payload.boo2 {
        sqlx::query!("UPDATE users SET boo2 = $1 WHERE id = $2", boo2, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserPinsPayload {
    progress: Option<Vec<i64>>,
    awards: Option<Vec<i64>>,
    // packet captures don't have profile pins in responses
    #[serde(skip_serializing)]
    profile_pins: Option<Vec<i64>>,
}

pub async fn get_my_pins(pool: Data<Arc<Pool<Postgres>>>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let user = sqlx::query!(
        "SELECT progress, awards FROM users WHERE id = $1", session.user_id
    )
    .fetch_optional(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or(error::ErrorNotFound("User not found"))?;

    Ok(Json(UserPinsPayload {
        progress: Some(user.progress),
        awards: Some(user.awards),
        profile_pins: None,
    }))
}

pub async fn update_my_pins(mut payload: Json<UserPinsPayload>, pool: Data<Arc<Pool<Postgres>>>, session: ReqData<SessionData>) -> Result<impl Responder> {
    if let Some(pins) = &payload.profile_pins {
        if pins.len() > 3 {
            return Err(error::ErrorBadRequest("Invalid profile pins list"));
        }
    }

    if let Some(awards) = &payload.awards {
        sqlx::query!("UPDATE users SET awards = $1 WHERE id = $2", awards, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(progress) = &payload.progress {
        sqlx::query!("UPDATE users SET progress = $1 WHERE id = $2", progress, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }
    if let Some(pins) = &payload.profile_pins {
        sqlx::query!("UPDATE users SET profile_pins = $1 WHERE id = $2", pins, session.user_id)
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }

    // packet captures don't have profile pins in the response
    payload.profile_pins = None;

    Ok(Json(payload))
}