use axum::response::{IntoResponse, Response};
use http::StatusCode;
use uuid::Uuid;

use crate::AppState;

pub fn db_error(error: sqlx::Error) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
}

pub async fn get_id_from_username(username: &str, state: &AppState) -> Result<Uuid, Response> {
    Ok(
        sqlx::query!("SELECT id FROM users WHERE online_id = $1", username)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found").into_response())?
            .id
    )
}

pub async fn check_slot(
    slot_id: i64,
    state: &AppState,
) -> Result<(), Response> {
    let slot_exists = sqlx::query!("SELECT EXISTS(SELECT id FROM slots WHERE id = $1)", slot_id)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?
        .exists
        .unwrap();

    if !slot_exists {
        return Err((StatusCode::NOT_FOUND, "Slot not found").into_response())
    }

    Ok(())
}

pub async fn check_slot_author(
    slot_id: i64,
    user_id: Uuid,
    state: &AppState,
) -> Result<(), Response> {
    let is_author = sqlx::query!(
        "SELECT author = $2 AS is_author FROM slots WHERE id = $1",
        slot_id,
        user_id,
    )
        .fetch_optional(&state.pool)
        .await
        .map_err(db_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Slot not found").into_response())?
        .is_author
        .unwrap();

    if !is_author {
        return Err((StatusCode::UNAUTHORIZED, "Cannot modify another user's slot").into_response())
    }

    Ok(())
}

pub async fn is_slot_hearted(
    user_id: Uuid,
    slot_id: i64,
    state: &AppState,
) -> Result<bool, Response> {
    Ok(
        sqlx::query!(
            "SELECT EXISTS(
                SELECT timestamp FROM favourite_slots
                WHERE user_id = $1 AND slot_id = $2
            )",
            user_id, slot_id
        )
            .fetch_one(&state.pool)
            .await
            .map_err(db_error)?
            .exists
            .unwrap()
    )
}

pub async fn is_slot_queued(
    user_id: Uuid,
    slot_id: i64,
    state: &AppState,
) -> Result<bool, Response> {
    Ok(
        sqlx::query!(
            "SELECT EXISTS(
                SELECT timestamp FROM queued_slots
                WHERE user_id = $1 AND slot_id = $2
            )",
            user_id, slot_id
        )
            .fetch_one(&state.pool)
            .await
            .map_err(db_error)?
            .exists
            .unwrap()
    )
}