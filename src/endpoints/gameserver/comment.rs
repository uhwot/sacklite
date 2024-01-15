use axum::{Router, routing::{get, post}, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, Extension};
use axum_extra::extract::Query;
use futures::TryStreamExt;
use maud::html as xml;
use serde::Deserialize;
use sqlx::QueryBuilder;
use sqlx::types::chrono::NaiveDateTime;

use crate::{extractors::Xml, types::SessionData, AppState};
use crate::endpoints::gameserver::comment::CommentTarget::{Slot, User};
use crate::endpoints::gameserver::SlotType;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/comments/:slot_type/:slot_id", get(slot_comments))
        .route("/postComment/:slot_type/:slot_id", post(post_slot_comment))
        .route("/deleteComment/:slot_type/:slot_id", post(delete_comment))
        .route("/userComments/:online_id", get(user_comments))
        .route("/postUserComment/:online_id", post(post_user_comment))
        .route("/deleteUserComment/:online_id", post(delete_comment))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentListQuery {
    page_start: i64,
    page_size: i64,
}

enum CommentTarget {
    User(String),
    Slot(SlotType, i64),
}

#[derive(sqlx::FromRow)]
struct Comment {
    id: i64,
    posted_at: NaiveDateTime,
    content: String,
    deleted_by_mod: bool,
    author_oid: String,
    deleter_oid: Option<String>,
}

async fn slot_comments(
    // TODO: use slot type
    Path((slot_type, slot_id)): Path<(SlotType, i64)>,
    query: Query<CommentListQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    comments(
        CommentTarget::Slot(slot_type, slot_id),
        query,
        state,
    ).await
}

async fn user_comments(
    Path(online_id): Path<String>,
    query: Query<CommentListQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    comments(
        CommentTarget::User(online_id),
        query,
        state,
    ).await
}

async fn comments(
    target: CommentTarget,
    params: Query<CommentListQuery>,
    state: AppState,
) -> Result<impl IntoResponse, Response> {
    // what the fuck have i done
    let mut query = QueryBuilder::new(
        "SELECT comm.id, comm.posted_at, comm.content, comm.deleted_by_mod,
        author.online_id AS author_oid,
        deleter.online_id AS deleter_oid
        FROM comments comm
        JOIN users author ON comm.author = author.id"
    );
    if let CommentTarget::User(_) = target {
        query.push(" JOIN users target_user ON comm.target_user = target_user.id");
    }
    query.push(" LEFT JOIN users AS deleter ON comm.deleted_by = deleter.id");
    match target {
        CommentTarget::Slot(_, id) => {
            query.push(" WHERE target_slot = ");
            query.push_bind(id);
        }
        CommentTarget::User(username) => {
            query.push(" WHERE target_user.online_id = ");
            query.push_bind(username);
        }
    }

    query.push(" ORDER BY comm.posted_at DESC");
    query.push(" LIMIT ");
    query.push_bind(params.page_size);
    query.push(" OFFSET ");
    query.push_bind(params.page_start - 1);

    let mut comments = query.build_query_as::<Comment>()
        .fetch(&state.pool);

    Ok(Xml(xml!(
        comments {
            @while let Some(comment) = comments.try_next().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())? {
                comment {
                    id { (comment.id) }
                    npHandle { (comment.author_oid) }
                    timestamp { (comment.posted_at.timestamp_millis()) }
                    @if comment.deleted_by_mod {
                        deleted { "true" }
                        deletedBy { "moderator" }
                        deleteType { "moderator" }
                    } @else if let Some(deleter_oid) = comment.deleter_oid {
                        deleted { "true" }
                        deletedBy { (deleter_oid) }
                        deleteType { "user" }
                    } @else {
                        message { (comment.content) }
                    }
                    thumbsup { "0" } // TODO: fix this once ratings are implemented
                    thumbsdown { "0" }
                    yourthumb { "0" }
                }
            }
        }
    )))
}

#[derive(Deserialize)]
struct PostCommentPayload {
    message: String,
}

async fn post_slot_comment(
    // TODO: use slot type
    Path((slot_type, slot_id)): Path<(SlotType, i64)>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
    payload: Xml<PostCommentPayload>,
) -> Result<impl IntoResponse, Response> {
    post_comment(
        CommentTarget::Slot(slot_type, slot_id),
        state,
        session,
        payload,
    ).await
}

async fn post_user_comment(
    Path(username): Path<String>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
    payload: Xml<PostCommentPayload>,
) -> Result<impl IntoResponse, Response> {
    post_comment(
        CommentTarget::User(username),
        state,
        session,
        payload,
    ).await
}

async fn post_comment(
    // TODO: use slot type
    target: CommentTarget,
    state: AppState,
    session: Extension<SessionData>,
    payload: Xml<PostCommentPayload>,
) -> Result<impl IntoResponse, Response> {
    let user_id = match target {
        CommentTarget::Slot(_, id) => {
            sqlx::query!("SELECT id FROM slots WHERE id = $1", id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
                .ok_or_else(|| (StatusCode::NOT_FOUND, "Slot not found").into_response())?;
            None
        },
        CommentTarget::User(ref username) => Some(
            sqlx::query!("SELECT id FROM users WHERE online_id = $1", username)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
                .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found").into_response())?
                .id
            )
    };

    match target {
        CommentTarget::Slot(_, id) => sqlx::query!(
            "INSERT INTO comments (author, target_slot, content) VALUES ($1, $2, $3)",
            session.user_id,
            id,
            payload.message
        ),
        CommentTarget::User(_) => sqlx::query!(
            "INSERT INTO comments (author, target_user, content) VALUES ($1, $2, $3)",
            session.user_id,
            user_id.unwrap(),
            payload.message
        )
    }
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentDeleteQuery {
    comment_id: i64,
}

async fn delete_comment(
    query: Query<CommentDeleteQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    let target_refs = sqlx::query!(
        "SELECT target_user IS NOT NULL AS user,
        target_slot IS NOT NULL AS slot
        FROM comments WHERE id = $1",
        query.comment_id
    )
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Comment not found").into_response())?;

    let target = if target_refs.slot.unwrap() {
        Slot(SlotType::User, 0)
    } else if target_refs.user.unwrap() {
        User(String::new())
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    };

    let is_deleted = sqlx::query!(
        "SELECT deleted_by IS NOT NULL OR deleted_by_mod = true AS is_deleted
        FROM comments WHERE id = $1",
        query.comment_id
    )
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .is_deleted
        .unwrap();

    if is_deleted {
        return Err((StatusCode::UNAUTHORIZED, "Comment already deleted").into_response());
    }

    let is_allowed = match target {
        CommentTarget::Slot(_, _) => sqlx::query!(
            "SELECT comments.author = $2 OR slots.author = $2 AS is_allowed
            FROM comments JOIN slots ON target_slot = slots.id
            WHERE comments.id = $1",
            query.comment_id,
            session.user_id,
        )
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
            .is_allowed
            .unwrap(),
        CommentTarget::User(_) => sqlx::query!(
            "SELECT comments.author = $2 OR target_user = $2 AS is_allowed
            FROM comments WHERE id = $1",
            query.comment_id,
            session.user_id,
        )
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
            .is_allowed
            .unwrap(),
    };

    if !is_allowed {
        return Err((StatusCode::UNAUTHORIZED, "Not allowed to delete comment").into_response());
    }

    sqlx::query!(
        "UPDATE comments SET deleted_by = $1
        FROM slots
        WHERE comments.id = $2",
        session.user_id,
        query.comment_id
    )
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    Ok(StatusCode::OK)
}
