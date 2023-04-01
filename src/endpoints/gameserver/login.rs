use std::time::{SystemTime, UNIX_EPOCH};

use actix_session::Session;
use actix_web::{Responder, web, Result, error};
use log::debug;
use maud::html as xml;

use crate::{
    responder::Xml,
    types::{
        npticket::NpTicket,
        pub_key_store::PubKeyStore,
        config::Config,
    },
};

pub async fn login(config: web::Data<Config>, pub_key_store: web::Data<PubKeyStore>, bytes: web::Bytes, session: Session) -> Result<impl Responder> {
    let npticket = NpTicket::parse_from_bytes(bytes).map_err(|e| {
        debug!("{e}");
        error::ErrorBadRequest("NpTicket parsing failed")
    })?;

    if config.verify_npticket_expiry {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if npticket.body.expire_date as u128 <= now {
            return Err(error::ErrorUnauthorized("NpTicket is expired"));
        }
    }

    if config.verify_npticket_signature {
        let sig_verified = npticket.verify_signature(&pub_key_store).map_err(|e| {
            debug!("{e}");
            error::ErrorBadRequest("NpTicket signature parsing failed")
        })?;

        if !sig_verified {
            debug!("key_id: {:?}, signature: {:?}", npticket.footer.key_id, npticket.footer.signature);
            return Err(error::ErrorUnauthorized("NpTicket signature doesn't match data and/or key"));
        }
    }

    // TODO: implement sessions

    session.insert("online_id", npticket.body.online_id).unwrap();

    Ok(Xml(
        xml! {
            loginResult {
                authTicket { "MM_AUTH=fuck" }
                lbpEnvVer { "sacklite" }
            }
        }.into_string()
    ))
}