use actix_web::{web, Result, Responder, error};
use maud::html as xml;

use crate::{DbPool, db::actions::get_user_by_online_id, responder::Xml, types::SessionData};

pub async fn user(path: web::Path<String>, pool: web::Data<DbPool>, session: web::ReqData<SessionData>) -> Result<impl Responder> {
    let online_id = path.into_inner();

    let user = web::block(move || {
        let mut conn = pool.get().unwrap();
        get_user_by_online_id(&mut conn, &online_id)
    })
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_err(error::ErrorInternalServerError)?;

    let user = user.ok_or(error::ErrorNotFound(""))?;

    Ok(Xml(xml!(
        user type="user" {
            npHandle icon="ass" { (user.online_id) } // TODO: icon hash here
        }
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
        yay2 { "0" }
        boo2 { "0" }
        biography { "fuck you" }
        reviewCount { "0" }
        commentCount { "0" }
        photosByMeCount { "0" }
        photosWithMeCount { "0" }
        commentsEnabled { "true" }
        location {
            x { "0" }
            y { "0" }
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
    ).into_string()))
}