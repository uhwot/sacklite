use axum::{Router, routing::post, extract::{State, Path}, Extension, response::{IntoResponse, Response}};
use http::StatusCode;

use crate::{AppState, types::SessionData, utils::db::{db_error, check_slot, is_slot_hearted}};

use super::SlotType;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/favourite/slot/:type/:id", post(favourite_slot))
        .route("/unfavourite/slot/:type/:id", post(unfavourite_slot))
}

async fn favourite_slot(
    State(state): State<AppState>,
    session: Extension<SessionData>,
    // TODO: handle slot type
    Path((_, slot_id)): Path<(SlotType, i64)>
) -> Result<impl IntoResponse, Response> {
    check_slot(slot_id, &state).await?;
    if is_slot_hearted(session.user_id, slot_id, &state).await? {
        return Err((StatusCode::UNAUTHORIZED, "Slot is already hearted").into_response())
    }

    sqlx::query!("INSERT INTO favourite_slots (user_id, slot_id) VALUES ($1, $2)", session.user_id, slot_id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;

    Ok(())
}

async fn unfavourite_slot(
    State(state): State<AppState>,
    session: Extension<SessionData>,
    // TODO: handle slot type
    Path((_, slot_id)): Path<(SlotType, i64)>
) -> Result<impl IntoResponse, Response> {
    if !is_slot_hearted(session.user_id, slot_id, &state).await? {
        return Err((StatusCode::UNAUTHORIZED, "Slot is not hearted").into_response())
    }

    sqlx::query!("DELETE FROM favourite_slots WHERE user_id = $1 AND slot_id = $2", session.user_id, slot_id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;

    Ok(())
}