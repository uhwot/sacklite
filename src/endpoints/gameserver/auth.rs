use std::time::{SystemTime, UNIX_EPOCH};

use actix_http::HttpMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{Responder, web, Result, error, HttpRequest, HttpResponse};
use log::{debug, warn};
use maud::html as xml;

use crate::{
    responder::Xml,
    types::{
        npticket::NpTicket,
        pub_key_store::PubKeyStore,
        config::Config, platform::Platform,
    },
    DbPool, db::actions::*,
};

pub async fn login(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    pub_key_store: web::Data<PubKeyStore>,
    payload: web::Bytes,
    session: Session
) -> Result<impl Responder> {
    let npticket = NpTicket::parse_from_bytes(payload).map_err(|e| {
        warn!("NpTicket parsing failed");
        debug!("{e}");
        error::ErrorBadRequest("")
    })?;

    if config.verify_npticket_expiry {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if npticket.body.expire_date as u128 <= now {
            warn!("NpTicket is expired");
            return Err(error::ErrorUnauthorized(""));
        }
    }

    if config.verify_npticket_signature {
        let sig_verified = npticket.verify_signature(&pub_key_store).map_err(|e| {
            warn!("NpTicket signature parsing failed");
            debug!("{e}");
            error::ErrorBadRequest("")
        })?;

        if !sig_verified {
            warn!("NpTicket signature doesn't match data and/or key");
            debug!("key_id: {:?}, signature: {:?}", npticket.footer.key_id, npticket.footer.signature);
            return Err(error::ErrorUnauthorized(""));
        }
    }

    let user = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        get_user_by_online_id(&mut conn, &npticket.body.online_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    
    if let Some(user) = user {
        let linked_id = match npticket.footer.key_id {
            Platform::PSN => user.psn_id,
            Platform::RPCN => user.rpcn_id,
        }.ok_or(error::ErrorUnauthorized(""))?;

        if linked_id != npticket.body.user_id as i64 {
            return Err(error::ErrorUnauthorized(""));
        }
    }

    

    //session.insert("linked_user_id", LinkedUserId::from_npticket(&npticket).to_string()).unwrap();
    //Identity::login(&req.extensions(), npticket.body.online_id).unwrap();

    Ok(Xml(
        xml! {
            loginResult {
                // this is replaced in the session hack middleware
                authTicket { "ass" }
                lbpEnvVer { "sacklite" }
            }
        }.into_string()
    ))
}

pub async fn goodbye(_: Identity, session: Session) -> Result<impl Responder> {
    session.purge();
    Ok(HttpResponse::Ok())
}