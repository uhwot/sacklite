use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use actix_session::Session;
use actix_web::{
    error,
    web::{Bytes, Data},
    HttpResponse, Responder, Result,
};
use maud::html as xml;
use sqlx::{types::BigDecimal, Pool, Postgres};
use uuid::Uuid;

use crate::{
    responder::Xml,
    types::{Config, GameVersion, NpTicket, Platform, PubKeyStore, SessionData},
};

pub async fn login(
    pool: Data<Arc<Pool<Postgres>>>,
    config: Data<Config>,
    pub_key_store: Data<PubKeyStore>,
    payload: Bytes,
    session: Session,
) -> Result<impl Responder> {
    let npticket = NpTicket::parse_from_bytes(payload).map_err(error::ErrorBadRequest)?;

    if config.verify_npticket_expiry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if npticket.body.expire_date as u128 <= now {
            return Err(error::ErrorUnauthorized("NpTicket is expired"));
        }
    }

    if config.verify_npticket_signature {
        let sig_verified = npticket
            .verify_signature(&pub_key_store)
            .map_err(error::ErrorBadRequest)?;

        if !sig_verified {
            return Err(error::ErrorUnauthorized(
                "NpTicket signature doesn't match data and/or key",
            ));
        }
    }

    let session_data = get_session_data(pool, npticket, &config).await?;

    let platform = session_data.platform as u8;
    let game_version = session_data.game_version as u8;

    session
        .insert("user_id", session_data.user_id.to_string())
        .unwrap();
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

struct UserData {
    id: Uuid,
    online_id: String,
}

async fn get_session_data(
    pool: Data<Arc<Pool<Postgres>>>,
    npticket: NpTicket,
    config: &Data<Config>,
) -> Result<SessionData> {
    let game_version =
        GameVersion::from_service_id(&npticket.body.service_id).map_err(error::ErrorBadRequest)?;

    let npticket_uid = BigDecimal::from(npticket.body.user_id);

    let user = match npticket.footer.key_id {
        Platform::Psn => {
            sqlx::query_as!(
                UserData,
                "SELECT id, online_id FROM users WHERE psn_id = $1",
                npticket_uid
            )
            .fetch_optional(&***pool)
            .await
        }
        Platform::Rpcn => {
            sqlx::query_as!(
                UserData,
                "SELECT id, online_id FROM users WHERE rpcn_id = $1",
                npticket_uid
            )
            .fetch_optional(&***pool)
            .await
        }
    }
    .map_err(error::ErrorInternalServerError)?;

    if let Some(user) = user {
        if user.online_id != npticket.body.online_id {
            if !config.rename_users_automatically {
                return Err(error::ErrorUnauthorized(
                    "Online ID doesn't match with user on server",
                ));
            }

            sqlx::query!(
                "UPDATE users SET online_id = $1 WHERE id = $2",
                npticket.body.online_id,
                user.id
            )
            .execute(&***pool)
            .await
            .map_err(error::ErrorInternalServerError)?;

            match npticket.footer.key_id {
                Platform::Psn => {
                    sqlx::query!("UPDATE users SET rpcn_id = NULL WHERE id = $1", user.id)
                        .execute(&***pool)
                        .await
                }
                Platform::Rpcn => {
                    sqlx::query!("UPDATE users SET psn_id = NULL WHERE id = $1", user.id)
                        .execute(&***pool)
                        .await
                }
            }
            .map_err(error::ErrorInternalServerError)?;
        }

        return Ok(SessionData {
            user_id: user.id,
            online_id: npticket.body.online_id,
            platform: npticket.footer.key_id,
            game_version,
        });
    }

    // TODO: check if user with the same online id exists

    if !config.create_user_on_connect {
        return Err(error::ErrorUnauthorized("User doesn't exist"));
    }

    let user_id = sqlx::query!(
        "INSERT INTO users (id, online_id) VALUES (gen_random_uuid(), $1) RETURNING id",
        npticket.body.online_id
    )
    .fetch_one(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .id;

    match npticket.footer.key_id {
        Platform::Psn => {
            sqlx::query!(
                "UPDATE users SET psn_id = $1 WHERE id = $2",
                npticket_uid,
                user_id
            )
            .execute(&***pool)
            .await
        }
        Platform::Rpcn => {
            sqlx::query!(
                "UPDATE users SET rpcn_id = $1 WHERE id = $2",
                npticket_uid,
                user_id
            )
            .execute(&***pool)
            .await
        }
    }
    .map_err(error::ErrorInternalServerError)?;

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
