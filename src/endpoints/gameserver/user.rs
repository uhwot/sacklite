use actix_web::{web, Result, Responder, error, HttpResponse};
use maud::html as xml;
use serde::Deserialize;

use crate::{DbPool, db::actions::{DbError, user::*}, responder::Xml, types::{SessionData, GameVersion}};

use super::Location;

pub async fn user(path: web::Path<String>, pool: web::Data<DbPool>, session: web::ReqData<SessionData>) -> Result<impl Responder> {
    let online_id = path.into_inner();

    let user = web::block(move || {
        let mut conn = pool.get().unwrap();
        get_user_by_online_id(&mut conn, &online_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let user = user.ok_or(error::ErrorNotFound(""))?;

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
            meh2 { (user.meh2) }
            boo2 { (user.boo2) }
            biography { (user.biography) }
            reviewCount { "0" }
            commentCount { "0" }
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
            pins {} // list of nums, ex: 2807907583,1675480291,1252477949
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
#[serde(rename_all = "camelCase")]
pub struct UpdateUser {
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

pub async fn update_user(payload: actix_xml::Xml<UpdateUser>, pool: web::Data<DbPool>, session: web::ReqData<SessionData>) -> Result<impl Responder> {
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