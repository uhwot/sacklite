use std::time::{SystemTime, UNIX_EPOCH};

use actix_identity::Identity;
use actix_session::Session;
use actix_web::{error, web, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use log::{debug, warn};
use maud::html as xml;
use thiserror::Error;

use crate::{
    db::actions::*,
    responder::Xml,
    types::{config::Config, npticket::NpTicket, platform::Platform, pub_key_store::PubKeyStore},
    DbPool,
};

pub async fn login(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    pub_key_store: web::Data<PubKeyStore>,
    payload: web::Bytes,
    session: Session,
) -> Result<impl Responder> {
    let npticket = NpTicket::parse_from_bytes(payload).map_err(|e| {
        warn!("NpTicket parsing failed");
        debug!("{e}");
        error::ErrorBadRequest("")
    })?;

    if config.verify_npticket_expiry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
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
            debug!(
                "key_id: {:?}, signature: {:?}",
                npticket.footer.key_id, npticket.footer.signature
            );
            return Err(error::ErrorUnauthorized(""));
        }
    }

    let user = web::block(move || get_login_user(&pool, npticket, &config)).await?;

    let (uuid, online_id, platform) = match user {
        Ok(user) => Ok(user),
        Err(e) => match e {
            LoginError::UserError => Err(error::ErrorUnauthorized("")),
            LoginError::DbError(_) => Err(error::ErrorInternalServerError("")),
        },
    }?;

    let platform: &str = platform.into();

    Identity::login(&req.extensions(), uuid).unwrap();
    session.insert("online_id", online_id).unwrap();
    session.insert("platform", platform).unwrap();

    Ok(Xml(xml! {
        loginResult {
            // this is replaced in the session hack middleware
            authTicket { "ass" }
            lbpEnvVer { "sacklite" }
        }
    }
    .into_string()))
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("database error")]
    DbError(#[from] DbError),
    #[error("user error")]
    UserError,
}

fn get_login_user(
    pool: &web::Data<DbPool>,
    npticket: NpTicket,
    config: &web::Data<Config>,
) -> std::result::Result<(String, String, Platform), LoginError> {
    let mut conn = pool.get().expect("Couldn't get db connection from pool");

    let user =
        get_user_by_online_id(&mut conn, &npticket.body.online_id).map_err(LoginError::DbError)?;

    if let Some(user) = user {
        let linked_id = match npticket.footer.key_id {
            Platform::Psn => user.psn_id,
            Platform::Rpcn => user.rpcn_id,
        }
        .ok_or(LoginError::UserError)?;

        if linked_id != npticket.body.user_id as i64 {
            return Err(LoginError::UserError);
        }

        return Ok((user.id, user.online_id, npticket.footer.key_id));
    }

    if !config.create_user_on_connect {
        return Err(LoginError::UserError);
    }

    let uuid = insert_new_user(&mut conn, &npticket.body.online_id).map_err(LoginError::DbError)?;
    match npticket.footer.key_id {
        Platform::Psn => set_user_psn_id(&mut conn, uuid, Some(npticket.body.user_id as i64))
            .map_err(LoginError::DbError)?,
        Platform::Rpcn => set_user_rpcn_id(&mut conn, uuid, Some(npticket.body.user_id as i64))
            .map_err(LoginError::DbError)?,
    };

    Ok((
        uuid.to_string(),
        npticket.body.online_id,
        npticket.footer.key_id,
    ))
}

pub async fn goodbye(_: Identity, session: Session) -> Result<impl Responder> {
    session.purge();
    Ok(HttpResponse::Ok())
}
