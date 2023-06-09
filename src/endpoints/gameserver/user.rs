use actix_web::{web::{self, Json, Data, ReqData, Path}, Result, Responder, error, HttpResponse};
use anyhow::Context;
use maud::html as xml;
use serde::{Deserialize, Serialize};

use crate::{DbPool, db::{actions::{DbError, user::*, comment::get_user_comment_count}, models::User}, responder::Xml, types::{SessionData, GameVersion}};

use super::Location;

pub async fn user(path: Path<String>, pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let online_id = path.into_inner();

    let (user, comment_count) = web::block(move || {
        let mut conn = pool.get().unwrap();
        
        let user = get_user_by_online_id(&mut conn, &online_id)?.context("User not found")?;
        let comment_count = get_user_comment_count(&mut conn, user.id)?;
        Ok::<(User, i64), DbError>((user, comment_count))
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(Xml(xml!(
        user type="user" {
            npHandle icon=(user.icon) { (user.online_id) }
            game { (session.game_version.to_num()) }
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
            commentCount { (comment_count) }
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
                (pins)
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

pub async fn users(query: web::Query<UsersQuery>, pool: Data<DbPool>) -> Result<impl Responder> {
    let users = web::block(move || {
        let mut conn = pool.get().unwrap();

        let mut users = Vec::new();

        for oid in &query.u {
            let user = get_user_by_online_id(&mut conn, &oid)?;
            if let Some(u) = user {
                users.push(u);
            }
        }

        Ok::<Vec<User>, DbError>(users)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(Xml(xml!(
        users {
            @for user in users {
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

pub async fn update_user(payload: actix_xml::Xml<UpdateUserPayload>, pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get().unwrap();
        let uid = session.user_id;

        if let Some(location) = &payload.location {
            set_user_location(&mut conn, uid, location.x, location.y)?;
        }
        if let Some(bio) = &payload.biography {
            set_user_biography(&mut conn, uid, bio)?;
        }
        if let Some(icon) = &payload.icon {
            set_user_icon(&mut conn, uid, icon)?;
        }
        if let Some(planets) = &payload.planets {
            match session.game_version {
                GameVersion::Lbp1 => {},
                GameVersion::Lbp2 => set_user_lbp2_planets(&mut conn, uid, planets)?,
                GameVersion::Lbp3 => set_user_lbp3_planets(&mut conn, uid, planets)?,
            }
        }
        if let Some(ccplanet) = &payload.cross_control_planet {
            set_user_ccplanet(&mut conn, uid, ccplanet)?;
        }
        if let Some(yay2) = &payload.yay2 {
            set_user_yay2(&mut conn, uid, yay2)?;
        }
        if let Some(meh2) = &payload.meh2 {
            set_user_meh2(&mut conn, uid, meh2)?;
        }
        if let Some(boo2) = &payload.boo2 {
            set_user_boo2(&mut conn, uid, boo2)?;
        }

        Ok::<(), DbError>(())
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

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

pub async fn get_my_pins(pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get().unwrap();

        get_user_by_uuid(&mut conn, session.user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let user = user.ok_or(error::ErrorInternalServerError(""))?;

    Ok(Json(UserPinsPayload {
        progress: Some(user.progress),
        awards: Some(user.awards),
        profile_pins: None,
    }))
}

pub async fn update_my_pins(payload: Json<UserPinsPayload>, pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    if let Some(pins) = &payload.profile_pins {
        if pins.len() > 3 {
            return Err(error::ErrorBadRequest(""));
        }
    }

    let mut payload_resp = payload.clone();

    web::block(move || {
        let mut conn = pool.get().unwrap();

        if let Some(awards) = &payload.awards {
            set_user_awards(&mut conn, session.user_id, awards)?;
        }
        if let Some(progress) = &payload.progress {
            set_user_progress(&mut conn, session.user_id, progress)?;
        }
        if let Some(pins) = &payload.profile_pins {
            set_user_profile_pins(&mut conn, session.user_id, pins)?;
        }

        Ok::<(), DbError>(())
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    // packet captures don't have profile pins in the response
    payload_resp.profile_pins = None;

    Ok(Json(payload_resp))
}