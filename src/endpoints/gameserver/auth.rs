use std::time::{SystemTime, UNIX_EPOCH};

use actix_session::Session;
use actix_web::{error, web::{self, Data, Bytes}, HttpResponse, Responder, Result};
use log::{debug, error};
use maud::html as xml;
use thiserror::Error;

use crate::{
    db::actions::{DbError, user::*},
    responder::Xml,
    types::{Config, GameVersion, NpTicket, Platform, PubKeyStore, SessionData},
    DbPool,
};

pub async fn login(
    pool: Data<DbPool>,
    config: Data<Config>,
    pub_key_store: Data<PubKeyStore>,
    payload: Bytes,
    session: Session,
) -> Result<impl Responder> {
    let npticket = NpTicket::parse_from_bytes(payload).map_err(|e| {
        debug!("NpTicket parsing failed");
        debug!("{e}");
        error::ErrorBadRequest("")
    })?;

    debug!("{npticket:#?}");

    if config.verify_npticket_expiry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if npticket.body.expire_date as u128 <= now {
            debug!("NpTicket is expired");
            return Err(error::ErrorUnauthorized(""));
        }
    }

    if config.verify_npticket_signature {
        let sig_verified = npticket.verify_signature(&pub_key_store).map_err(|e| {
            debug!("NpTicket signature parsing failed");
            debug!("{e}");
            error::ErrorBadRequest("")
        })?;

        if !sig_verified {
            debug!("NpTicket signature doesn't match data and/or key");
            debug!(
                "key_id: {:?}, signature: {:?}",
                npticket.footer.key_id, npticket.footer.signature
            );
            return Err(error::ErrorUnauthorized(""));
        }
    }

    let session_data = web::block(move || get_session_data(&pool, npticket, &config)).await?;

    let session_data = match session_data {
        Ok(d) => Ok(d),
        Err(e) => match e {
            LoginError::UserError => Err(error::ErrorUnauthorized("")),
            LoginError::DbError(e) => {
                error!("Database error: {e}");
                Err(error::ErrorInternalServerError(""))
            }
        },
    }?;

    let platform: &str = session_data.platform.into();
    let game_version: &str = session_data.game_version.into();

    session.insert("user_id", session_data.user_id.to_string()).unwrap();
    session.insert("online_id", session_data.online_id).unwrap();
    session.insert("platform", platform).unwrap();
    session.insert("game_version", game_version).unwrap();

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

fn get_session_data(
    pool: &Data<DbPool>,
    npticket: NpTicket,
    config: &Data<Config>,
) -> std::result::Result<SessionData, LoginError> {
    let mut conn = pool.get().expect("Couldn't get db connection from pool");

    let game_version = match GameVersion::from_service_id(&npticket.body.service_id) {
        Ok(ver) => ver,
        Err(_) => return Err(LoginError::UserError),
    };

    let user = match npticket.footer.key_id {
        Platform::Psn => get_user_by_psn_id(&mut conn, npticket.body.user_id),
        Platform::Rpcn => get_user_by_rpcn_id(&mut conn, npticket.body.user_id),
    }.map_err(LoginError::DbError)?;

    if let Some(user) = user {
        if user.online_id != npticket.body.online_id {
            if !config.rename_users_automatically {
                return Err(LoginError::UserError);
            }

            set_user_online_id(&mut conn, user.id, &npticket.body.online_id).map_err(LoginError::DbError)?;

            match npticket.footer.key_id {
                Platform::Psn => set_user_rpcn_id(&mut conn, user.id, None),
                Platform::Rpcn => set_user_psn_id(&mut conn, user.id, None),
            }.map_err(LoginError::DbError)?;
        }

        return Ok(SessionData {
            user_id: user.id,
            online_id: npticket.body.online_id,
            platform: npticket.footer.key_id,
            game_version,
        });
    }

    if !config.create_user_on_connect {
        return Err(LoginError::UserError);
    }

    let user_id = insert_new_user(&mut conn, &npticket.body.online_id).map_err(LoginError::DbError)?;
    match npticket.footer.key_id {
        Platform::Psn => set_user_psn_id(&mut conn, user_id, Some(npticket.body.user_id))
            .map_err(LoginError::DbError)?,
        Platform::Rpcn => set_user_rpcn_id(&mut conn, user_id, Some(npticket.body.user_id))
            .map_err(LoginError::DbError)?,
    };

    Ok(SessionData {
        user_id,
        online_id: npticket.body.online_id,
        platform: npticket.footer.key_id,
        game_version,
    })
}

pub async fn goodbye(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok()
}
