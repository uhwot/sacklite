use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::State, body::Bytes, response::{IntoResponse, Response}, http::StatusCode};
use maud::html as xml;
use sqlx::types::BigDecimal;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    extractors::Xml,
    types::{GameVersion, NpTicket, Platform, SessionData}, AppState, utils::db::db_error,
};

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    payload: Bytes,
) -> Result<impl IntoResponse, Response> {
    let npticket = NpTicket::parse_from_bytes(payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;

    if state.config.verify_npticket_expiry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if npticket.body.expire_date as u128 <= now {
            return Err((StatusCode::UNAUTHORIZED, "NpTicket is expired").into_response());
        }
    }

    if state.config.verify_npticket_signature {
        let sig_verified = npticket
            .verify_signature()
            .map_err(|_| StatusCode::BAD_REQUEST.into_response())?;

        if !sig_verified {
            return Err((
                StatusCode::UNAUTHORIZED,
                "NpTicket signature doesn't match data and/or key",
            ).into_response());
        }
    }

    let session_data = get_session_data(state, npticket).await?;

    let platform = session_data.platform as u8;
    let game_version = session_data.game_version as u8;

    session.cycle_id().await.unwrap();
    session
        .insert("user_id", session_data.user_id.to_string())
        .await
        .unwrap();
    session.insert("online_id", session_data.online_id).await.unwrap();
    session.insert("platform", platform).await.unwrap();
    session.insert("game_version", game_version).await.unwrap();

    Ok(Xml(xml! {
        loginResult {
            authTicket { (format!("MM_AUTH={}", session.id().unwrap())) }
            lbpEnvVer { "sacklite" }
        }
    }))
}

pub async fn goodbye(session: Session) -> impl IntoResponse {
    session.delete()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
}

struct UserData {
    id: Uuid,
    online_id: String,
}

async fn get_session_data(
    state: AppState,
    npticket: NpTicket,
) -> Result<SessionData, Response> {
    let game_version =
        GameVersion::from_service_id(&npticket.body.service_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;

    let npticket_uid = BigDecimal::from(npticket.body.user_id);

    let user = match npticket.footer.key_id {
        Platform::Psn => {
            sqlx::query_as!(
                UserData,
                "SELECT id, online_id FROM users WHERE psn_id = $1",
                npticket_uid
            )
            .fetch_optional(&state.pool)
            .await
        }
        Platform::Rpcn => {
            sqlx::query_as!(
                UserData,
                "SELECT id, online_id FROM users WHERE rpcn_id = $1",
                npticket_uid
            )
            .fetch_optional(&state.pool)
            .await
        }
    }
    .map_err(db_error)?;

    if let Some(user) = user {
        if user.online_id != npticket.body.online_id {
            if !state.config.rename_users_automatically {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Online ID doesn't match with user on server"
                ).into_response());
            }

            sqlx::query!(
                "UPDATE users SET online_id = $1 WHERE id = $2",
                npticket.body.online_id,
                user.id
            )
            .execute(&state.pool)
            .await
            .map_err(db_error)?;

            match npticket.footer.key_id {
                Platform::Psn => {
                    sqlx::query!("UPDATE users SET rpcn_id = NULL WHERE id = $1", user.id)
                        .execute(&state.pool)
                        .await
                }
                Platform::Rpcn => {
                    sqlx::query!("UPDATE users SET psn_id = NULL WHERE id = $1", user.id)
                        .execute(&state.pool)
                        .await
                }
            }
            .map_err(db_error)?;
        }

        return Ok(SessionData {
            user_id: user.id,
            online_id: npticket.body.online_id,
            platform: npticket.footer.key_id,
            game_version,
        });
    }

    // TODO: check if user with the same online id exists

    if !state.config.create_user_on_connect {
        return Err((StatusCode::UNAUTHORIZED, "User doesn't exist").into_response());
    }

    let user_id = sqlx::query!(
        "INSERT INTO users (id, online_id) VALUES (gen_random_uuid(), $1) RETURNING id",
        npticket.body.online_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?
    .id;

    match npticket.footer.key_id {
        Platform::Psn => {
            sqlx::query!(
                "UPDATE users SET psn_id = $1 WHERE id = $2",
                npticket_uid,
                user_id
            )
            .execute(&state.pool)
            .await
        }
        Platform::Rpcn => {
            sqlx::query!(
                "UPDATE users SET rpcn_id = $1 WHERE id = $2",
                npticket_uid,
                user_id
            )
            .execute(&state.pool)
            .await
        }
    }
    .map_err(db_error)?;

    Ok(SessionData {
        user_id,
        online_id: npticket.body.online_id,
        platform: npticket.footer.key_id,
        game_version,
    })
}
